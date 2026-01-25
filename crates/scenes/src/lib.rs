pub mod falling_particles;
pub mod scenes;

pub use falling_particles::FallingParticles;
pub use scenes::Scene;

use pp_physics::World;

//All scene types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneType {
    FallingParticles
    // LiquidSimulation for later
}

impl SceneType {
    //UI ausgabe von verfügbaren szenen
    pub fn all() -> &'static [SceneType] {
        &[
            SceneType::FallingParticles
            // SceneType::LiquidSimulation
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SceneType::FallingParticles => "Falling Particles",
            // SceneType::WaterSimulation => "Water Simulation",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SceneType::FallingParticles => "Particles falling down to customizable gravity"
            // SceneType::LiquidSimulation => TBD
        }
    }

    pub fn create_scene(&self, world: &mut World) -> Box<dyn Scene> {
        world.clear();

        match self {
            SceneType::FallingParticles => {
                FallingParticles::init_world(world);
                Box::new(FallingParticles::new())
            }
            // SceneType::LiquidSimulation => {
            //    LiquidSimulation::init_world(world);
            //    Box::new(LiquidSimulation::new())
            // }
        }
    }
}