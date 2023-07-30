use super::events::EventCollector;
use glam::Vec2;
use rapier2d::na::Vector2;
use rapier2d::prelude::*;
use rustc_hash::FxHashMap;
use std::f32::consts;

pub struct PhysicsContext {
    pub gravity: Vector2<f32>,
    pub rigidbodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub interpolation_data: FxHashMap<RigidBodyHandle, InterpolationData>,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joints: ImpulseJointSet,
    pub multibody_joints: MultibodyJointSet,
    pub solver: CCDSolver,
    pub hooks: Box<dyn PhysicsHooks>,
    pub events: EventCollector,
    pub running: bool,
}

pub struct InterpolationData {
    pub position_previous: Option<Vec2>,
    pub rotation_previous: Option<f32>,
    pub position_current: Vec2,
    pub rotation_current: f32,
}

impl PhysicsContext {
    pub fn new() -> Self {
        Self {
            gravity: Vector2::new(0.0, -9.81),
            rigidbodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            interpolation_data: Default::default(),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            solver: CCDSolver::new(),
            hooks: Box::new(()),
            events: Default::default(),
            running: true,
        }
    }

    pub fn step(&mut self, timestamp: f32) {
        self.events.clear();

        if !self.running {
            return;
        }

        self.integration_parameters.dt = timestamp;
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigidbodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.solver,
            None,
            self.hooks.as_ref(),
            &self.events,
        );

        for (handle, rigidbody) in self.rigidbodies.iter() {
            let translation = rigidbody.position().translation;
            let angle = rigidbody.position().rotation.angle();

            if let Some(interpolation_data) = self.interpolation_data.get_mut(&handle) {
                let position_previous = interpolation_data.position_current;
                let mut rotation_previous = interpolation_data.rotation_current;

                interpolation_data.position_current = translation.into();
                interpolation_data.rotation_current = angle;

                if interpolation_data.rotation_current - rotation_previous > consts::PI {
                    rotation_previous += 2.0 * consts::PI;
                }

                if interpolation_data.rotation_current - rotation_previous < -consts::PI {
                    rotation_previous -= 2.0 * consts::PI;
                }

                interpolation_data.position_previous = Some(position_previous);
                interpolation_data.rotation_previous = Some(rotation_previous);
            } else {
                self.interpolation_data.insert(handle, InterpolationData::new(translation.into(), angle));
            }
        }

        let mut orphans = Vec::new();

        for handle in &mut self.interpolation_data.keys() {
            if !self.rigidbodies.contains(*handle) {
                orphans.push(*handle);
            }
        }

        for handle in &orphans {
            self.interpolation_data.remove(handle);
        }
    }
}

impl Default for PhysicsContext {
    fn default() -> Self {
        Self::new()
    }
}

impl InterpolationData {
    pub fn new(position: Vec2, rotation: f32) -> Self {
        Self { position_previous: None, rotation_previous: None, position_current: position, rotation_current: rotation }
    }

    pub fn get_position_interpolated(&self, alpha: f32) -> Vec2 {
        if let Some(position_previous) = self.position_previous {
            self.position_current * alpha + position_previous * (1.0 - alpha)
        } else {
            self.position_current
        }
    }

    pub fn get_rotation_interpolated(&self, alpha: f32) -> f32 {
        if let Some(rotation_previous) = self.rotation_previous {
            self.rotation_current * alpha + rotation_previous * (1.0 - alpha)
        } else {
            self.rotation_current
        }
    }

    pub fn clear(&mut self) {
        self.position_previous = None;
        self.rotation_previous = None;
        self.position_current = Vec2::new(0.0, 0.0);
        self.rotation_current = 0.0;
    }
}
