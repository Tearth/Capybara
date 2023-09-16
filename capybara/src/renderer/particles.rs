use super::context::RendererContext;
use super::sprite::Sprite;
use super::sprite::TextureType;
use crate::utils::rand::NewRand;
use crate::utils::storage::Storage;
use arrayvec::ArrayVec;
use glam::Vec2;
use glam::Vec4;
use instant::Instant;
use log::error;
use std::f32::consts;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

#[derive(Default)]
pub struct ParticleEmitter<const WAYPOINTS: usize> {
    pub position: Vec2,
    pub size: Vec2,
    pub period: f32,
    pub amount: u32,
    pub bursts: Option<u32>,
    pub interpolation: ParticleInterpolation,

    pub particle_size: Option<Vec2>,
    pub particle_lifetime: f32,
    pub particle_texture_id: Option<usize>,
    pub particle_texture_type: TextureType,

    last_burst_time: Option<Instant>,

    pub particles: Storage<Particle<WAYPOINTS>>,
    pub velocity_waypoints: ArrayVec<ParticleParameter<Vec2>, WAYPOINTS>,
    pub rotation_waypoints: ArrayVec<ParticleParameter<f32>, WAYPOINTS>,
    pub scale_waypoints: ArrayVec<ParticleParameter<Vec2>, WAYPOINTS>,
    pub color_waypoints: ArrayVec<ParticleParameter<Vec4>, WAYPOINTS>,
}

pub struct Particle<const WAYPOINTS: usize> {
    pub postion: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub size: Option<Vec2>,
    pub color: Vec4,
    pub birthday: Instant,
    pub lifetime: f32,

    pub velocity_variations: ArrayVec<Vec2, WAYPOINTS>,
    pub rotation_variations: ArrayVec<f32, WAYPOINTS>,
    pub scale_variations: ArrayVec<Vec2, WAYPOINTS>,
    pub color_variations: ArrayVec<Vec4, WAYPOINTS>,
}

pub struct ParticleParameter<T> {
    pub base: T,
    pub variation: T,
}

#[derive(Copy, Clone, Debug, Default)]
pub enum ParticleInterpolation {
    #[default]
    Linear,
    Cosine,
}

impl<const WAYPOINTS: usize> ParticleEmitter<WAYPOINTS> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn update(&mut self, now: Instant, delta: f32) {
        let mut fire = if let Some(last_burst_time) = self.last_burst_time {
            (now - last_burst_time).as_secs_f32() >= self.period
        } else {
            self.last_burst_time.is_none()
        };

        if let Some(bursts) = self.bursts {
            if bursts == 0 {
                fire = false;
            }
        }

        if fire {
            let offset = self.position - self.size / 2.0;

            for _ in 0..self.amount {
                let velocity_variations = generate_variations(&self.velocity_waypoints);
                let rotation_variations = generate_variations(&self.rotation_waypoints);
                let scale_variations = generate_variations(&self.scale_waypoints);
                let color_variations = generate_variations(&self.color_waypoints);

                self.particles.store(Particle {
                    postion: Vec2::new(fastrand::f32(), fastrand::f32()) * self.size + offset,
                    rotation: 0.0,
                    scale: Vec2::new(1.0, 1.0),
                    size: self.particle_size,
                    color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                    birthday: now,
                    lifetime: self.particle_lifetime,
                    velocity_variations,
                    rotation_variations,
                    scale_variations,
                    color_variations,
                });
            }

            self.last_burst_time = Some(now);

            if let Some(bursts) = &mut self.bursts {
                *bursts -= 1;
            }
        }

        let mut removed_ids = Vec::new();
        for (index, particle) in self.particles.iter_enumerate_mut() {
            let particle_time = (now - particle.birthday).as_secs_f32();

            if particle_time >= particle.lifetime {
                removed_ids.push(index);
            } else {
                let factor = particle_time / particle.lifetime;

                let p = calculate(factor, &self.velocity_waypoints, &particle.velocity_variations, particle.postion, self.interpolation);
                let r = calculate(factor, &self.rotation_waypoints, &particle.rotation_variations, particle.rotation, self.interpolation);
                let s = calculate(factor, &self.scale_waypoints, &particle.scale_variations, particle.scale, self.interpolation);
                let c = calculate(factor, &self.color_waypoints, &particle.color_variations, particle.color, self.interpolation);

                particle.postion += p * delta;
                particle.rotation = r;
                particle.scale = s;
                particle.color = c;
            }
        }

        for index in removed_ids {
            self.particles.remove(index);
        }
    }

    pub fn draw(&mut self, renderer: &mut RendererContext) {
        let mut sprite = Sprite::new();
        sprite.texture_id = self.particle_texture_id;
        sprite.texture_type = self.particle_texture_type.clone();

        if sprite.is_animation() {
            error!("Animations in particles aren't supported");
        }

        for particle in self.particles.iter() {
            sprite.position = particle.postion;
            sprite.rotation = particle.rotation;
            sprite.scale = particle.scale;
            sprite.size = particle.size;
            sprite.color = particle.color;

            renderer.draw_sprite(&sprite);
        }
    }

    pub fn is_finished(&self) -> bool {
        self.particles.is_empty()
    }
}

impl<T> ParticleParameter<T> {
    pub fn new(base: T, variation: T) -> ParticleParameter<T> {
        ParticleParameter { base, variation }
    }
}

fn generate_variations<T, const WAYPOINTS: usize>(waypoints: &ArrayVec<ParticleParameter<T>, WAYPOINTS>) -> ArrayVec<T, WAYPOINTS>
where
    T: Copy + NewRand<T> + Sub<Output = T> + Mul<T, Output = T> + Div<f32, Output = T>,
{
    let mut variations = ArrayVec::new();
    for waypoint in waypoints {
        variations.push(waypoint.variation / 2.0 - waypoint.variation * T::new_rand(0.0..1.0));
    }

    variations
}

fn calculate<T>(lifetime_factor: f32, waypoints: &[ParticleParameter<T>], variations: &[T], default: T, interpolation: ParticleInterpolation) -> T
where
    T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>,
{
    match waypoints.len() {
        0 => default,
        1 => waypoints[0].base + variations[0],
        _ => {
            let count = (waypoints.len() - 1) as f32;
            let lifetime_per_waypoint = 1.0 / count;
            let waypoint_index = (count * lifetime_factor) as usize;
            let waypoint_offset = (lifetime_factor % lifetime_per_waypoint) * count;

            let waypoint_a = &waypoints[waypoint_index];
            let waypoint_b = &waypoints[waypoint_index + 1];

            interpolate(
                waypoint_a.base + variations[waypoint_index],
                waypoint_b.base + variations[waypoint_index + 1],
                waypoint_offset,
                interpolation,
            )
        }
    }
}

fn interpolate<T>(from: T, to: T, offset: f32, interpolation: ParticleInterpolation) -> T
where
    T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>,
{
    let offset = match interpolation {
        ParticleInterpolation::Linear => offset,
        ParticleInterpolation::Cosine => ((offset * consts::PI - consts::PI / 2.0).sin() + 1.0) / 2.0,
    };

    from + (to - from) * offset
}
