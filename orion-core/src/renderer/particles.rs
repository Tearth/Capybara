use super::context::RendererContext;
use super::sprite::Sprite;
use crate::utils::rand::NewRand;
use glam::Vec2;
use glam::Vec4;
use instant::Instant;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

pub struct Particle {
    pub postion: Vec2,
    pub size: Option<Vec2>,
    pub color: Vec4,
    pub birthday: Instant,
    pub lifetime: f32,

    pub color_variations: Vec<Vec4>,
    pub velocity_variations: Vec<Vec2>,
}

pub struct ParticleParameter<T> {
    pub base: T,
    pub variation: T,
}

#[derive(Default)]
pub struct ParticleEmitter {
    pub position: Vec2,
    pub size: Vec2,
    pub period: f32,
    pub bursts: u32,
    pub amount: u32,

    pub particle_size: Option<Vec2>,
    pub particle_lifetime: f32,

    last_burst_time: Option<Instant>,

    pub particles: Vec<Particle>,
    pub color_waypoints: Vec<ParticleParameter<Vec4>>,
    pub velocity_waypoints: Vec<ParticleParameter<Vec2>>,
}

impl ParticleEmitter {
    pub fn update(&mut self, now: Instant, delta: f32) {
        let fire = if let Some(last_burst_time) = self.last_burst_time {
            (now - last_burst_time).as_secs_f32() >= self.period
        } else {
            self.last_burst_time.is_none()
        };

        if fire {
            self.last_burst_time = Some(now);
        }

        if fire {
            let offset = self.position - self.size / 2.0;

            for _ in 0..self.amount {
                let color_variations = generate_variations(&self.color_waypoints);
                let velocity_variations = generate_variations(&self.velocity_waypoints);

                self.particles.push(Particle {
                    postion: Vec2::new(fastrand::f32(), fastrand::f32()) * self.size + offset,
                    size: self.particle_size,
                    color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                    birthday: now,
                    lifetime: self.particle_lifetime,
                    color_variations,
                    velocity_variations,
                })
            }
        }

        let mut particles_to_remove = Vec::new();

        for (index, particle) in self.particles.iter_mut().enumerate() {
            let particle_time = (now - particle.birthday).as_secs_f32();

            if particle_time >= particle.lifetime {
                particles_to_remove.push(index);
            } else {
                let lifetime_factor = particle_time / particle.lifetime;

                particle.color = calculate(lifetime_factor, &self.color_waypoints, &particle.color_variations);
                particle.postion += calculate(lifetime_factor, &self.velocity_waypoints, &particle.velocity_variations) * delta;
            }
        }

        for index in particles_to_remove.iter().rev() {
            self.particles.remove(*index);
        }
    }

    pub fn draw(&mut self, renderer: &mut RendererContext) {
        let mut sprite = Sprite::new();
        for particle in &self.particles {
            sprite.position = particle.postion;
            sprite.size = particle.size;
            sprite.color = particle.color;

            renderer.draw_sprite(&sprite).unwrap();
        }
    }
}

impl<T> ParticleParameter<T> {
    pub fn new(base: T, variation: T) -> ParticleParameter<T> {
        ParticleParameter { base, variation }
    }
}

fn generate_variations<T>(waypoints: &Vec<ParticleParameter<T>>) -> Vec<T>
where
    T: Copy + NewRand<T> + Sub<Output = T> + Mul<T, Output = T> + Div<f32, Output = T>,
{
    let mut variations = Vec::new();
    for waypoint in waypoints {
        variations.push(waypoint.variation / 2.0 - waypoint.variation * T::new_rand());
    }

    variations
}

fn calculate<T>(lifetime_factor: f32, waypoints: &[ParticleParameter<T>], variations: &[T]) -> T
where
    T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>,
{
    let lifetime_per_waypoint = 1.0 / (waypoints.len() - 1) as f32;
    let waypoint_index = ((waypoints.len() - 1) as f32 * lifetime_factor) as usize;
    let waypoint_offset = (waypoints.len() - 1) as f32 * (lifetime_factor % lifetime_per_waypoint);

    let waypoint_a = &waypoints[waypoint_index];
    let waypoint_b = &waypoints[waypoint_index + 1];

    interpolate(waypoint_a.base + variations[waypoint_index], waypoint_b.base + variations[waypoint_index + 1], waypoint_offset)
}

fn interpolate<T>(from: T, to: T, offset: f32) -> T
where
    T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>,
{
    from + (to - from) * offset
}
