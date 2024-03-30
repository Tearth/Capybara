use crate::*;
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use byteorder::WriteBytesExt;
use capybara::glam::IVec2;
use capybara::physics::context::PhysicsContext;
use capybara::powder::chunk::ParticleData;
use capybara::powder::chunk::ParticleState;
use capybara::powder::simulation::PowderSimulation;
use capybara::renderer::context::RendererContext;
use std::fs::File;

pub fn load(
    path: &str,
    simulation: &mut PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>,
    renderer: &mut RendererContext,
    physics: &mut PhysicsContext,
) {
    let mut file = File::open(path).unwrap();
    let mut particles_count = file.read_u32::<LittleEndian>().unwrap();

    simulation.reset(renderer, physics);

    while particles_count > 0 {
        let position = IVec2::new(file.read_i32::<LittleEndian>().unwrap(), file.read_i32::<LittleEndian>().unwrap());
        let r#type = file.read_u32::<LittleEndian>().unwrap() as usize;
        let hpressure = file.read_f32::<LittleEndian>().unwrap();

        if r#type != usize::MAX {
            let definitions = simulation.definitions.clone();
            let definition = &definitions.read().unwrap()[r#type];
            simulation
                .add_particle(position, ParticleData { r#type, state: definition.state, color: definition.color, hpressure, ..Default::default() });
        }

        particles_count -= 1;
    }
}

pub fn save(path: &str, simulation: &mut PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>) {
    let mut file = File::create(path).unwrap();
    file.write_u32::<LittleEndian>(simulation.particles_count);

    for chunk in simulation.chunks.values() {
        for particle in &chunk.particles {
            if particle.present {
                let particle = match particle.state {
                    ParticleState::Solid => chunk.solid.get(particle.id),
                    ParticleState::Powder => chunk.powder.get(particle.id),
                    ParticleState::Fluid => chunk.fluid.get(particle.id),
                    _ => panic!("Invalid particle state ({:?})", particle.state),
                }
                .unwrap();

                let particle = simulation.get_particle(particle.position).unwrap();
                file.write_i32::<LittleEndian>(particle.position.x).unwrap();
                file.write_i32::<LittleEndian>(particle.position.y).unwrap();
                file.write_u32::<LittleEndian>(particle.r#type as u32).unwrap();
                file.write_f32::<LittleEndian>(particle.hpressure).unwrap();
            }
        }
    }
}
