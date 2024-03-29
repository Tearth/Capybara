use self::chunk::Chunk;
use self::features::gravity;
use self::features::liquidity;
use self::features::velocity;
use super::*;
use crate::error_return;
use crate::glam::IVec2;
use crate::glam::Vec2;
use crate::glam::Vec4;
use crate::physics::context::PhysicsContext;
use crate::rapier2d::dynamics::RigidBodyHandle;
use crate::renderer::context::RendererContext;
use crate::rustc_hash::FxHashMap;
use crate::rustc_hash::FxHashSet;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::RwLock;

pub struct PowderSimulation<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> {
    pub definitions: Rc<RwLock<Vec<ParticleDefinition>>>,
    pub chunks: FxHashMap<IVec2, Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>>,
    pub structures: Vec<Structure>,

    pub gravity: Vec2,
    pub particles_count: u32,
}

#[derive(Clone, Default)]
pub struct Structure {
    pub rigidbody_handle: RigidBodyHandle,
    pub particle_indices: Vec<(StructureData, IVec2)>,
    pub temporary_positions: Vec<IVec2>,
    pub center: Vec2,
}

#[derive(Clone)]
pub enum StructureData {
    Position(IVec2),
    Particle(Rc<RefCell<ParticleData>>),
}

impl<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER> {
    pub fn logic(&mut self, renderer: &mut RendererContext, physics: &mut PhysicsContext, delta: f32) {
        self.process_solid();
        self.process_powder(delta);
        self.process_fluid(delta);

        for (chunk_position, chunk) in &mut self.chunks {
            if !chunk.initialized {
                chunk.initialize(renderer, *chunk_position);
            }

            if chunk.dirty {
                chunk.update(physics);
            }
        }
    }

    pub fn draw(&mut self, renderer: &mut RendererContext) {
        for chunk in &mut self.chunks.values_mut() {
            chunk.draw(renderer);
        }
    }

    pub fn process_solid(&mut self) {}

    pub fn process_powder(&mut self, delta: f32) {
        let definitions = self.definitions.clone();
        let definitions = definitions.read().unwrap();

        for key in self.chunks.keys().cloned().collect::<Vec<_>>() {
            let mut last_id = None;
            while let Some(id) = self.chunks[&key].powder.get_next_id(last_id) {
                let particle = self.chunks[&key].powder.get_unchecked(id).clone();
                let particle_borrow = particle.as_ref().borrow();
                drop(particle_borrow);

                gravity::simulate(self, &definitions, particle.clone(), delta);
                velocity::simulate(self, particle, delta);
                last_id = Some(id);
            }
        }
    }

    pub fn process_fluid(&mut self, delta: f32) {
        let definitions = self.definitions.clone();
        let definitions = definitions.read().unwrap();

        for key in self.chunks.keys().cloned().collect::<Vec<_>>() {
            let mut last_id = None;
            while let Some(id) = self.chunks[&key].fluid.get_next_id(last_id) {
                let particle = self.chunks[&key].fluid.get_unchecked(id).clone();
                let particle_borrow = particle.as_ref().borrow();
                drop(particle_borrow);

                gravity::simulate(self, &definitions, particle.clone(), delta);
                velocity::simulate(self, particle, delta);
                last_id = Some(id);
            }

            let mut current_substep = 0;
            let mut processed_particles = 0;

            loop {
                let mut last_id = None;
                while let Some(id) = self.chunks[&key].fluid.get_next_id(last_id) {
                    let center_particle = self.chunks[&key].fluid.get_unchecked(id).clone();
                    let center_particle_borrow = center_particle.as_ref().borrow();
                    let definition = &definitions[center_particle_borrow.r#type];
                    drop(center_particle_borrow);

                    if current_substep < definition.fluidity {
                        liquidity::simulate(self, &definitions, center_particle);
                        processed_particles += 1;
                    }

                    last_id = Some(id);
                }

                // No more fluid particles with enough fluidity to perform next substep
                if processed_particles == 0 {
                    break;
                }

                current_substep += 1;
                processed_particles = 0;
            }
        }
    }

    pub fn reset(&mut self, renderer: &mut RendererContext, physics: &mut PhysicsContext) {
        for structure in &self.structures {
            physics.rigidbodies.remove(
                structure.rigidbody_handle,
                &mut physics.island_manager,
                &mut physics.colliders,
                &mut physics.impulse_joints,
                &mut physics.multibody_joints,
                true,
            );
        }

        // TODO: clear colliders
        // TODO: free canvas textures

        for chunk_position in self.chunks.keys().cloned().collect::<Vec<IVec2>>() {
            self.remove_chunk(renderer, chunk_position);
        }

        self.chunks.clear();
        self.structures.clear();
    }

    pub fn add_chunk(&mut self, chunk_position: IVec2) {
        if self.chunks.contains_key(&chunk_position) {
            error_return!("Chunk with position {} already exists", chunk_position);
        }

        self.chunks.insert(chunk_position, Chunk::default());
    }

    pub fn remove_chunk(&mut self, renderer: &mut RendererContext, chunk_position: IVec2) {
        if let Some(chunk) = self.chunks.get(&chunk_position) {
            renderer.textures.remove(chunk.canvas.texture_id);
            self.chunks.remove(&chunk_position);
        } else {
            error_return!("Chunk with position {} does not exists", chunk_position);
        }
    }

    pub fn get_chunk(&self, position: IVec2) -> Option<&Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>> {
        self.chunks.get(&(self.get_chunk_key(position)))
    }

    pub fn get_chunk_mut(&mut self, position: IVec2) -> Option<&mut Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>> {
        self.chunks.get_mut(&(self.get_chunk_key(position)))
    }

    fn get_chunk_key(&self, position: IVec2) -> IVec2 {
        let mut chunk_position = IVec2::new(position.x >> 6, position.y >> 6);

        if position.x < 0 {
            chunk_position.x -= 1;
        }
        if position.y < 0 {
            chunk_position.y -= 1;
        }

        chunk_position
    }

    pub fn add_particle(&mut self, position: IVec2, particle: Rc<RefCell<ParticleData>>) {
        if self.get_chunk_mut(position).is_none() {
            self.add_chunk(self.get_chunk_key(position));
        }

        self.get_chunk_mut(position).unwrap_or_else(|| panic!("Chunk not found")).add_particle(position, particle);
        self.particles_count += 1;
    }

    pub fn remove_particle(&mut self, position: IVec2) -> Option<Rc<RefCell<ParticleData>>> {
        if let Some(particle) = self.get_chunk_mut(position).unwrap_or_else(|| panic!("Chunk not found")).remove_particle(position) {
            self.particles_count -= 1;
            Some(particle)
        } else {
            None
        }
    }

    pub fn move_particle(&mut self, from_position: IVec2, to_position: IVec2) {
        let particle = self.remove_particle(from_position).unwrap_or_else(|| panic!("Particle not found"));
        self.add_particle(to_position, particle);
    }

    pub fn swap_particles(&mut self, from_position: IVec2, to_position: IVec2) {
        let from_particle = self.remove_particle(from_position).unwrap_or_else(|| panic!("Particle not found"));
        let to_particle = self.remove_particle(to_position).unwrap_or_else(|| panic!("Particle not found"));

        self.add_particle(from_position, to_particle);
        self.add_particle(to_position, from_particle);
    }

    pub fn particle_exists(&self, position: IVec2) -> bool {
        if let Some(chunk) = self.get_chunk(position) {
            chunk.particle_exists(position)
        } else {
            false
        }
    }

    pub fn get_particle(&self, position: IVec2) -> Option<Rc<RefCell<ParticleData>>> {
        self.get_chunk(position).and_then(|p| p.get_particle(position))
    }

    pub fn set_particle_color(&mut self, position: IVec2, color: Vec4) {
        self.get_chunk_mut(position).unwrap_or_else(|| panic!("Chunk not found")).set_particle_color(position, color);
    }

    pub fn create_structure(&mut self, physics: &mut PhysicsContext, position: IVec2, points: &mut FxHashSet<IVec2>) {
        for point in points.iter() {
            if let Some(particle) = self.get_particle(*point) {
                particle.borrow_mut().structure = true;
            }
        }

        let structure = physics::create_structure::<PARTICLE_SIZE, PIXELS_PER_METER>(physics, position, points);
        self.structures.push(structure);

        // let rigidbody_handle = physics::create_structure::<PARTICLE_SIZE, PIXELS_PER_METER>(physics, position, &mut points);
        // let particle_indices = points.iter().map(|p| (StructureData::Position(*p), *p)).collect::<Vec<(StructureData, IVec2)>>();
        // Structure { rigidbody_handle, particle_indices, temporary_positions: Vec::new(), center: position.as_vec2() }
    }

    pub fn displace_fluid(&mut self, position: IVec2, forbidden: &FxHashSet<IVec2>) {
        let particle_center = self.get_particle(position).expect("Particle is not a fluid").clone();
        let particle_center = particle_center.borrow_mut();

        let particle_type = particle_center.r#type;
        let mut available_neighbours = Vec::new();

        for neighbour_offset in [IVec2::new(1, 0), IVec2::new(-1, 0), IVec2::new(0, 1), IVec2::new(0, -1)] {
            let neighbour_position = particle_center.position + neighbour_offset;
            let particle_neighbour = self.get_particle(neighbour_position);

            if forbidden.contains(&neighbour_position) {
                continue;
            }

            if let Some(particle_neighbour) = particle_neighbour {
                if particle_center.r#type == particle_neighbour.as_ref().borrow().r#type {
                    available_neighbours.push((neighbour_position, false));
                }
            } else {
                available_neighbours.push((neighbour_position, true));
            }
        }

        let average_hpressure = particle_center.hpressure / available_neighbours.len() as f32;

        if !available_neighbours.is_empty() {
            for (neighbour_position, empty) in available_neighbours {
                if empty {
                    self.add_particle(
                        neighbour_position,
                        Rc::new(RefCell::new(ParticleData {
                            r#type: particle_type,
                            state: ParticleState::Fluid,
                            color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                            hpressure: average_hpressure,
                            ..Default::default()
                        })),
                    );
                } else {
                    self.get_particle(neighbour_position).unwrap().borrow_mut().hpressure += average_hpressure;
                }
            }
        }

        self.remove_particle(position);
    }

    pub fn update_structures(&mut self, physics: &mut PhysicsContext) {
        for s in 0..self.structures.len() {
            let mut particles_to_move = Vec::new();
            let mut forbidden_for_fluid = FxHashSet::default();

            let rigidbody = physics.rigidbodies.get(self.structures[0].rigidbody_handle).unwrap();
            let position =
                Vec2::new(rigidbody.position().translation.x, rigidbody.position().translation.y) * PIXELS_PER_METER as f32 / PARTICLE_SIZE as f32;
            let rotation = rigidbody.rotation().angle();

            for p in 0..self.structures[s].particle_indices.len() {
                match self.structures[s].particle_indices[p].clone().0 {
                    StructureData::Position(position) => {
                        forbidden_for_fluid.insert(position);
                        particles_to_move.push((self.remove_particle(position).unwrap(), self.structures[s].particle_indices[p].1));
                    }
                    StructureData::Particle(particle) => {
                        forbidden_for_fluid.insert(particle.as_ref().borrow().position);
                        particles_to_move.push((particle, self.structures[s].particle_indices[p].1));
                    }
                }
            }

            for p in 0..self.structures[s].temporary_positions.len() {
                self.remove_particle(self.structures[s].temporary_positions[p]).unwrap();
            }

            self.structures[s].particle_indices.clear();
            self.structures[s].temporary_positions.clear();
            let mut potential_holes = FxHashMap::default();

            for particle in &mut particles_to_move {
                let (particle, original_position) = particle.clone();

                let offset = original_position.as_vec2() - self.structures[s].center;
                let offset_after_rotation = Vec2::new(
                    offset.x * rotation.cos() - offset.y * rotation.sin(), // fmt
                    offset.x * rotation.sin() + offset.y * rotation.cos(),
                );
                let position = (position + offset_after_rotation).as_ivec2();

                let blocking_particle = self.get_particle(position);
                let blocking_particle_state = blocking_particle.clone().map(|p| p.as_ref().borrow().state).unwrap_or(ParticleState::Unknown);

                if blocking_particle.is_none() || blocking_particle_state == ParticleState::Fluid {
                    if blocking_particle.is_some() {
                        self.displace_fluid(position, &forbidden_for_fluid);
                    }

                    self.add_particle(position, particle);
                    self.structures[s].particle_indices.push((StructureData::Position(position), original_position));

                    for neighbour_offset in [IVec2::new(1, 0), IVec2::new(-1, 0), IVec2::new(0, 1), IVec2::new(0, -1)] {
                        let neighbour_position = position + neighbour_offset;
                        let neighbour_particle = self.get_particle(neighbour_position);
                        let neighbour_particle_state =
                            neighbour_particle.clone().map(|p| p.as_ref().borrow().state).unwrap_or(ParticleState::Unknown);

                        if neighbour_particle.is_none() || neighbour_particle_state == ParticleState::Fluid {
                            let key = neighbour_position.x | (neighbour_position.y << 16);
                            if let Some(data) = potential_holes.get_mut(&key) {
                                *data += 1;
                            } else {
                                potential_holes.insert(key, 1);
                            }
                        }
                    }
                } else {
                    self.structures[s].particle_indices.push((StructureData::Particle(particle), original_position));
                }
            }

            for (key, filled_sides) in &potential_holes {
                if *filled_sides == 4 {
                    let position = IVec2::new(key & 0xffff, key >> 16);
                    if let Some(particle) = self.get_particle(position) {
                        if particle.as_ref().borrow().state == ParticleState::Fluid {
                            self.displace_fluid(position, &forbidden_for_fluid);
                        }
                    }

                    let neighbour_particle = self.get_particle(position + IVec2::new(0, 1));
                    if let Some(neighbour_particle) = neighbour_particle {
                        let temporary_particle = neighbour_particle;
                        if !self.particle_exists(position) {
                            self.add_particle(position, temporary_particle);
                            self.structures[s].temporary_positions.push(position);
                        }
                    }
                }
            }
        }
    }

    pub fn apply_forces(&mut self, physics: &mut PhysicsContext) {
        for s in 0..self.structures.len() {
            let rigidbody = physics.rigidbodies.get_mut(self.structures[s].rigidbody_handle).unwrap();

            for p in 0..self.structures[s].particle_indices.len() {
                if let StructureData::Position(position) = self.structures[s].particle_indices[p].0 {
                    let mut hpressure = Vec2::ZERO;
                    let particle = self.get_particle(position).unwrap();
                    let particle = particle.as_ref().borrow();

                    for neighbour_offset in [IVec2::new(1, 0), IVec2::new(-1, 0), IVec2::new(0, 1), IVec2::new(0, -1)] {
                        let neighbour_position = particle.position + neighbour_offset;
                        let neighbour_particle = self.get_particle(neighbour_position);
                        let neighbour_particle_state =
                            neighbour_particle.clone().map(|p| p.as_ref().borrow().state).unwrap_or(ParticleState::Unknown);
                        let neighbour_particle_hpressure = neighbour_particle.clone().map(|p| p.as_ref().borrow().hpressure).unwrap_or(0.0);

                        if neighbour_particle_state == ParticleState::Fluid {
                            hpressure += -neighbour_offset.as_vec2() * neighbour_particle_hpressure;
                        }
                    }

                    if hpressure.length() > 1.0 {
                        let position = particle.position.as_vec2() * PARTICLE_SIZE as f32 + PARTICLE_SIZE as f32 / 2.0;
                        rigidbody.apply_impulse_at_point((hpressure * 0.08).into(), (position / PIXELS_PER_METER as f32).into(), true);
                    }
                }
            }
        }
    }

    pub fn is_position_valid(&self, position: IVec2) -> bool {
        self.get_chunk(position).is_some()
    }
}

impl<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> Default
    for PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>
{
    fn default() -> Self {
        Self {
            definitions: Default::default(),
            chunks: Default::default(),
            structures: Default::default(),
            gravity: Vec2::new(0.0, -160.0),
            particles_count: Default::default(),
        }
    }
}

impl ParticleData {
    pub fn present(&self) -> bool {
        self.r#type != usize::MAX
    }
}
