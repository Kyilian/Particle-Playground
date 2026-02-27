pub mod barnes_hut_nbody;
pub mod benchmark_scene;
pub mod falling_particles;
mod lavalamp_scene;
pub mod nbody_scene;
pub mod scenes;
pub mod test_scene;
pub use lavalamp_scene::LavaLampScene;

pub use barnes_hut_nbody::BarnesHutNbody;
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
    BarnesHutNbody,
    LavaLamp,
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
            SceneType::BarnesHutNbody,
            SceneType::LavaLamp,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SceneType::FallingParticles => "Falling Particles",
            SceneType::BenchmarkScene => "Benchmark Scene",
            SceneType::TestScene => "Test Scene",
            SceneType::NBodyScene => "N-Body Scene",
            // SceneType::WaterSimulation => "Water Simulation",
            SceneType::BarnesHutNbody => "Barnes Hut N-Body",
            SceneType::LavaLamp => "Lava Lamp",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SceneType::FallingParticles => "Particles falling down to customizable gravity",
            SceneType::BenchmarkScene => "Performance-Test: Continuously spawns particles until the fps fall under 30.",
            SceneType::TestScene => "Test-Scene with different collider-types (circle and triangle).",
            SceneType::NBodyScene => "N-Body gravitation simulation, left: single particle, right: particle-cluster, mmb: heavy mass."
            // SceneType::LiquidSimulation => TBD
            SceneType::BarnesHutNbody => "Barnes Hut N-Body",
            SceneType::LavaLamp => "Lava Lamp",
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
            SceneType::BarnesHutNbody => Box::new(BarnesHutNbody::new()),
            SceneType::LavaLamp => Box::new(LavaLampScene::new()),
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
