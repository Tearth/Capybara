use self::simulation::PowderSimulationDebugSettings;
use super::canvas::Canvas;
use super::*;
use crate::glam::IVec2;
use crate::glam::Vec4;
use crate::physics::context::PhysicsContext;
use crate::renderer::context::RendererContext;
use crate::renderer::shape::Shape;
use crate::utils::storage::Storage;
use rapier2d::geometry::ColliderHandle;
use rustc_hash::FxHashMap;

pub struct Chunk {
    pub initialized: bool,
    pub active: bool,
    pub dirty: bool,
    pub canvas: Canvas,
    pub solid_collider: Option<ColliderHandle>,
    pub position: IVec2,

    pub particles: Vec<ParticleIndex>,
    pub solid: Storage<ParticleData>,
    pub powder: Storage<ParticleData>,
    pub fluid: Storage<ParticleData>,

    pub(crate) chunk_size: i32,
    pub(crate) particle_size: i32,
    pub(crate) pixels_per_meter: i32,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ParticleIndex {
    pub id: usize,
    pub present: bool,
    pub state: ParticleState,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ParticleData {
    pub r#type: usize,
    pub state: ParticleState,
    pub structure: bool,
    pub position: IVec2,
    pub offset: Vec2,
    pub velocity: Vec2,
    pub color: Vec4,
    pub hpressure: f32,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ParticleState {
    #[default]
    Unknown,
    Solid,
    Powder,
    Fluid,
}

impl Chunk {
    pub fn new(chunk_size: i32, particle_size: i32, pixels_per_meter: i32) -> Self {
        Self {
            initialized: false,
            active: false,
            dirty: false,
            canvas: Canvas::new(chunk_size, particle_size),
            solid_collider: None,
            position: Default::default(),

            particles: [ParticleIndex::default()].repeat((chunk_size * chunk_size) as usize),
            solid: Default::default(),
            powder: Default::default(),
            fluid: Default::default(),

            chunk_size,
            particle_size,
            pixels_per_meter,
        }
    }

    pub fn initialize(&mut self, renderer: &mut RendererContext, chunk_position: IVec2) {
        self.canvas.initialize(renderer, chunk_position);
        self.position = chunk_position;
        self.initialized = true;
    }

    pub fn update(&mut self, physics: &mut PhysicsContext) {
        if let Some(handle) = self.solid_collider.take() {
            physics.colliders.remove(handle, &mut physics.island_manager, &mut physics.rigidbodies, false);
        }

        if !self.solid.is_empty() {
            let mut points = FxHashMap::default();
            for particle in self.solid.iter() {
                if !particle.structure {
                    points.insert(particle.position, 0.0);
                }
            }

            if let Some((mut collider, center)) = physics::create_collider(&mut points, None, self.particle_size, self.pixels_per_meter) {
                collider.set_translation(center.into());

                let handle = physics.colliders.insert(collider);
                self.solid_collider = Some(handle);
            }
        }

        self.dirty = false;
    }

    pub fn draw(&mut self, renderer: &mut RendererContext) {
        self.canvas.update_texture(renderer);
        self.canvas.draw(renderer);
    }

    pub fn draw_debug(&mut self, renderer: &mut RendererContext, settings: &PowderSimulationDebugSettings) {
        let size = Vec2::ONE * self.chunk_size as f32 * self.particle_size as f32;
        let left_bottom = self.position.as_vec2() * size;
        let right_top = (self.position + IVec2::ONE).as_vec2() * size;
        let color = if self.active { settings.chunk_active_color } else { settings.chunk_inactive_color };

        renderer.draw_shape(&Shape::new_frame(left_bottom, right_top, 1.0, color));
    }

    pub fn add_particle(&mut self, position: IVec2, mut particle: ParticleData) -> usize {
        let index = self.position_to_index(position);

        if self.particles[index].present {
            panic!("Particle already exists");
        }

        particle.position = position;

        let id = match particle.state {
            ParticleState::Solid => {
                self.dirty = true;
                self.solid.store(particle)
            }
            ParticleState::Powder => self.powder.store(particle),
            ParticleState::Fluid => self.fluid.store(particle),
            _ => panic!("Invalid particle state ({:?})", particle.state),
        };

        self.active = true;
        self.particles[index] = ParticleIndex { id, present: true, state: particle.state };
        self.canvas.set_particle(position, particle.color);

        id
    }

    pub fn remove_particle(&mut self, position: IVec2) -> Option<ParticleData> {
        let index = self.position_to_index(position);

        let (id, state) = if self.particles[index].present {
            (self.particles[index].id, self.particles[index].state)
        } else {
            return None;
        };

        let particle = match state {
            ParticleState::Solid => {
                self.dirty = true;
                self.solid.remove(id)
            }
            ParticleState::Powder => self.powder.remove(id),
            ParticleState::Fluid => self.fluid.remove(id),
            _ => return None,
        };

        self.active = true;
        self.particles[index].present = false;

        if particle.is_some() {
            self.canvas.set_particle(position, Vec4::new(0.0, 0.0, 0.0, 1.0));
        }

        particle
    }

    pub fn get_particle(&self, position: IVec2) -> Option<&ParticleData> {
        let index = self.position_to_index(position);
        let particle = self.particles[index];

        if particle.present {
            Some(self.get_storage(particle.state).get_unchecked(particle.id))
        } else {
            None
        }
    }

    pub fn get_particle_mut(&mut self, position: IVec2) -> Option<&mut ParticleData> {
        let index = self.position_to_index(position);
        let particle = self.particles[index];

        if particle.present {
            Some(self.get_storage_mut(particle.state).get_unchecked_mut(particle.id))
        } else {
            None
        }
    }

    pub fn get_storage(&self, state: ParticleState) -> &Storage<ParticleData> {
        match state {
            ParticleState::Solid => &self.solid,
            ParticleState::Powder => &self.powder,
            ParticleState::Fluid => &self.fluid,
            ParticleState::Unknown => panic!("Invalid storage"),
        }
    }

    pub fn get_storage_mut(&mut self, state: ParticleState) -> &mut Storage<ParticleData> {
        match state {
            ParticleState::Solid => &mut self.solid,
            ParticleState::Powder => &mut self.powder,
            ParticleState::Fluid => &mut self.fluid,
            ParticleState::Unknown => panic!("Invalid storage"),
        }
    }

    pub fn particle_exists(&self, position: IVec2) -> bool {
        self.particles[self.position_to_index(position)].present
    }

    pub fn set_particle_color(&mut self, position: IVec2, color: Vec4) {
        self.canvas.set_particle(position, color);
    }

    fn position_to_index(&self, position: IVec2) -> usize {
        ((position.x & (self.chunk_size - 1)) + (position.y & (self.chunk_size - 1)) * self.chunk_size) as usize
    }
}

pub fn get_chunk_key(position: IVec2) -> IVec2 {
    let mut chunk_position = IVec2::new(position.x >> 5, position.y >> 5);

    if position.x < 0 {
        chunk_position.x -= 1;
    }
    if position.y < 0 {
        chunk_position.y -= 1;
    }

    chunk_position
}
