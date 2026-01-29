pub mod benchmark_scene;
pub mod falling_particles;
pub mod nbody_scene;
pub mod scenes;
pub mod test_scene;
pub mod benchmark_scene;

pub use benchmark_scene::BenchmarkScene;
pub use falling_particles::FallingParticles;
pub use benchmark_scene::BenchmarkScene;
pub use nbody_scene::NBodyScene;
pub use scenes::Scene;
pub use test_scene::TestScene;
pub use scenes::Scene;

use pp_physics::World;

//All scene types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneType {
    FallingParticles,
    BenchmarkScene,
    TestScene,
    // LiquidSimulation for later
}

impl SceneType {
    //UI ausgabe von verfügbaren szenen
    pub fn all() -> &'static [SceneType] {
        &[
            SceneType::FallingParticles,
            SceneType::BenchmarkScene,
            SceneType::TestScene,
            // SceneType::LiquidSimulation
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SceneType::FallingParticles => "Falling Particles",
            SceneType::BenchmarkScene => "Benchmark Scene",
            SceneType::TestScene => "Test Scene",
            // SceneType::WaterSimulation => "Water Simulation",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SceneType::FallingParticles => "Particles falling down to customizable gravity",
            SceneType::BenchmarkScene => "Performance-Test: Spawnt kontinuierlich Partikel bis die FPS unter 30 fallen.",
            SceneType::TestScene => "Test-Szene mit verschiedenen Collider-Typen (Kreis und Rechteck).",
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
            // SceneType::LiquidSimulation => {
            //    LiquidSimulation::init_world(world);
            //    Box::new(LiquidSimulation::new())
            // }
        }
    }
}
