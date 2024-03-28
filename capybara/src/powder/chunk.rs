use super::canvas::Canvas;
use super::*;
use crate::glam::IVec2;
use crate::glam::Vec4;
use crate::renderer::context::RendererContext;
use crate::utils::storage::Storage;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Chunk<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32> {
    pub initialized: bool,
    pub canvas: Canvas<CHUNK_SIZE, PARTICLE_SIZE>,

    pub particles: Vec<ParticleIndex>,
    pub solid: Storage<Rc<RefCell<ParticleData>>>,
    pub powder: Storage<Rc<RefCell<ParticleData>>>,
    pub fluid: Storage<Rc<RefCell<ParticleData>>>,
}

impl<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32> Chunk<CHUNK_SIZE, PARTICLE_SIZE> {
    pub fn initialize(&mut self, renderer: &mut RendererContext, chunk_position: IVec2) {
        self.canvas.initialize(renderer, chunk_position);
        self.initialized = true;
    }

    pub fn draw(&mut self, renderer: &mut RendererContext) {
        self.canvas.update_texture(renderer);
        self.canvas.draw(renderer);
    }

    pub fn add_particle(&mut self, position: IVec2, particle: Rc<RefCell<ParticleData>>) -> usize {
        let index = self.position_to_index(position);

        if self.particles[index].present {
            panic!("Particle already exists");
        }

        particle.borrow_mut().position = position;

        let id = match particle.borrow().state {
            ParticleState::Solid => self.solid.store(particle.clone()),
            ParticleState::Powder => self.powder.store(particle.clone()),
            ParticleState::Fluid => self.fluid.store(particle.clone()),
            _ => panic!("Invalid particle state ({:?})", particle.borrow().state),
        };
        self.particles[index] = ParticleIndex { id, present: true, state: particle.borrow().state };
        self.canvas.set_particle(position, particle.borrow().color);

        id
    }

    pub fn remove_particle(&mut self, position: IVec2) -> Option<Rc<RefCell<ParticleData>>> {
        let index = self.position_to_index(position);

        let (id, state) = if self.particles[index].present {
            (self.particles[index].id, self.particles[index].state)
        } else {
            return None;
        };

        let particle = match state {
            ParticleState::Solid => self.solid.remove(id),
            ParticleState::Powder => self.powder.remove(id),
            ParticleState::Fluid => self.fluid.remove(id),
            _ => return None,
        };
        self.particles[index].present = false;

        if particle.is_some() {
            self.canvas.set_particle(position, Vec4::new(0.0, 0.0, 0.0, 1.0));
        }

        particle
    }

    pub fn get_particle(&self, position: IVec2) -> Option<Rc<RefCell<ParticleData>>> {
        let index = self.position_to_index(position);
        let particle = self.particles[index];

        if particle.present {
            match particle.state {
                ParticleState::Solid => Some(self.solid.get_unchecked(particle.id).clone()),
                ParticleState::Powder => Some(self.powder.get_unchecked(particle.id).clone()),
                ParticleState::Fluid => Some(self.fluid.get_unchecked(particle.id).clone()),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn particle_exists(&self, position: IVec2) -> bool {
        self.particles[self.position_to_index(position)].present
    }

    pub fn set_particle_color(&mut self, position: IVec2, color: Vec4) {
        self.canvas.set_particle(position, color);
    }

    fn position_to_index(&self, position: IVec2) -> usize {
        ((position.x & (CHUNK_SIZE - 1)) + (position.y & (CHUNK_SIZE - 1)) * CHUNK_SIZE) as usize
    }
}

impl<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32> Default for Chunk<CHUNK_SIZE, PARTICLE_SIZE> {
    fn default() -> Self {
        Self {
            initialized: false,
            canvas: Default::default(),
            particles: [ParticleIndex::default()].repeat((CHUNK_SIZE * CHUNK_SIZE) as usize),
            solid: Default::default(),
            powder: Default::default(),
            fluid: Default::default(),
        }
    }
}
