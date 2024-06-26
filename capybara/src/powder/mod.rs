use self::chunk::ParticleState;
use glam::Vec2;
use glam::Vec4;

pub mod canvas;
pub mod chunk;
pub mod features;
pub mod local;
pub mod physics;
pub mod simulation;
pub mod structures;

#[derive(Clone, Debug, Default)]
pub struct ParticleDefinition {
    /// Name displayed in the menu
    pub name: String,

    /// State of the particle (cannot be None)
    pub state: ParticleState,

    /// Base color of the particle before applying any modifiers
    pub color: Vec4,

    pub mass: f32,
    pub density: f32,
    pub displacement: f32,
    pub drag: f32,

    /// How much hydrostatic pressure can the particle hold compared to the one above without inflating (applies only to fluids) - larger
    /// value means it can hold bigger hydrostatic pressure in the equilibrium state
    pub compressibility: f32,

    /// How fast fluid will propagate hydrostatic pressure which directly transltes to movement speed - larger value means more substeps per tick
    pub fluidity: usize,

    // What hydrostatic pressure is needed for particle to inflate right and left
    pub extensibility: f32,

    pub hpressure_gradient_length: f32,
    pub hpressure_gradient_end: Vec4,
}
