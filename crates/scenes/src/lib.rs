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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            // SceneType::WaterSimulation => "Water Simulation",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SceneType::FallingParticles => "Particles falling down to customizable gravity",
            SceneType::BenchmarkScene => "Performance-Test: Continuously spawns particles until the fps fall under 30.",
            SceneType::TestScene => "Test-Scene with different collider-types (circle and triangle).",
            SceneType::NBodyScene => "N-Body gravitation simulation, left-click to spawn a particle with a stronger pull."
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
            SceneType::NBodyScene => {
                NBodyScene::init_world(world);
                Box::new(NBodyScene::new())
            } 
            // SceneType::LiquidSimulation => {
              //    LiquidSimulation::init_world(world);
              //    Box::new(LiquidSimulation::new())
              // }
        }
    }
}
