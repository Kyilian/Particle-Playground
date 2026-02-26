pub mod benchmark_scene;
pub mod falling_particles;
pub mod nbody_scene;
pub mod scenes;
pub mod test_scene;

pub use benchmark_scene::BenchmarkScene;
pub use falling_particles::FallingParticles;
pub use nbody_scene::NBodyScene;
pub use scenes::Scene;
pub use test_scene::TestScene;

use pp_physics::World;

//All scene types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneType {
    FallingParticles,
    BenchmarkScene,
    TestScene,
    NBodyScene, // LiquidSimulation for later
}

impl SceneType {
    //UI ausgabe von verfügbaren szenen
    pub fn all() -> &'static [SceneType] {
        &[
            SceneType::FallingParticles,
            SceneType::BenchmarkScene,
            SceneType::TestScene,
            SceneType::NBodyScene,
            // SceneType::LiquidSimulation
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SceneType::FallingParticles => "Falling Particles",
            SceneType::BenchmarkScene => "Benchmark Scene",
            SceneType::TestScene => "Test Scene",
            SceneType::NBodyScene => "N-Body Scene",
            // SceneType::LiquidSimulation => "Water Simulation",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SceneType::FallingParticles => "Particles falling down to customizable gravity",
            SceneType::BenchmarkScene => "Performance-Test: Continuously spawns particles until the fps fall under 30.",
            SceneType::TestScene => "Test-Scene with different collider-types (circle and triangle).",
            SceneType::NBodyScene => "N-Body gravitation simulation, left: single particle, right: particle-cluster, mmb: heavy mass."
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
            SceneType::BenchmarkScene => {
                BenchmarkScene::init_world(world);
                Box::new(BenchmarkScene::new())
            }
            SceneType::TestScene => {
                TestScene::init_world(world);
                Box::new(TestScene::new())
            }
            SceneType::NBodyScene => Box::new(NBodyScene::new()), // SceneType::LiquidSimulation => {
                                                                  //    LiquidSimulation::init_world(world);
                                                                  //    Box::new(LiquidSimulation::new())
                                                                  // }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //check if all scene types have valid display names and descriptions
    #[test]
    fn test_scene_type() {
        for scene_type in SceneType::all() {
            //name not empty
            let name = scene_type.display_name();
            assert!(!name.is_empty(), "A SceneType doesn't have a name.");

            //description not empty
            let description = scene_type.description();
            assert!(
                !description.is_empty(),
                "A SceneType doesn't have a description."
            );

            //description minimum value
            assert!(
                description.len() >= 10,
                "A scene description is too small (under 10 signs)."
            );
        }
    }
}
