# Particle Playground

Welcome to the Particle Playground, a 2D sandbox build in Rust. The project uses Verlet integration for physics, GPU-instanced rendering with wgpu, and implements the Barnes-Hut algorithm for efficient gravitational N-body simulation. Built as part of the Rust Software Engineering Project at LMU (Winter Term 25/26).

## Getting Started
1. Install [Rust](https://www.rust-lang.org/tools/install)
2. Clone the repository
```bash
git clone https://github.com/Kyilian/Particle-Playground.git
```
3. Open the folder in a terminal
```bash
cd Particle-Playground
```
4. Run `cargo run --release`
6. Choose a scene
5. Enjoy

![Launcher Screenshot](/assets/launcher.png)

![Ingame Screenshot](/assets/game.png)

## Showcase

[![Particle Playground Showcase](https://img.youtube.com/vi/QHup1YOwdOk/maxresdefault.jpg)](https://www.youtube.com/watch?v=QHup1YOwdOk)

## Project Structure

The project is organized as a Cargo workspace with four crates:

| Crate | Description |
|-------|-------------|
| `app` | Window management, event loop, egui integration |
| `physics` | Verlet integration, collisions, gravity, quadtree |
| `render` | GPU rendering via wgpu with instanced particle drawing |
| `scenes` | Individual simulations implementing the `Scene` trait |

## Scenes

### 1. Falling Particles

The main playground scene. Spawn particles that fall under gravity and bounce inside a circular collider.

**Controls:**
- **Left Click** — Spawn mode dependent (see UI radio buttons):
  - *Single:* Spawns one particle at the cursor
  - *Cluster:* Spawns a random cluster of particles around the cursor
  - *Magnet:* Places a magnet attractor/repulsor at the cursor
- **Right Click** — Finds and prints the nearest particle to the cursor
- **Middle Click** — Randomizes the particle color (when not using heatmap)
- **Scroll Wheel** — Resizes the nearest magnet or adjusts its strength (toggle in UI)

**UI Features:**
- Gravity, collider radius, and particle size sliders
- Magnet size and strength sliders
- Left click mode selection (Single / Cluster / Magnet)
- Scroll mode selection (Resize Magnet / Adjust Magnet Strength)
- Heatmap toggle: colors particles by velocity (blue = slow, red = fast). This feature is exclusive to this scene.
- Color picker for fixed particle color when heatmap is off
- "Remove Magnets" button to clear all magnets

---

### 2. Benchmark Scene

A stress test that continuously spawns particles until the frame rate drops below 30 FPS, giving a clear measure of how many particles the engine can handle.

**Controls:**
- **Start/Stop** button or **Enter** key to toggle the benchmark
- No mouse interaction in the simulation area

**UI Features:**
- Particles per second, start speed, gravity, collider radius, and particle radius sliders
- Live FPS counter (green above 30, red below)
- Auto-stops at < 30 FPS

---

### 3. Test Scene

A development sandbox for testing different collider types and physics parameters.

**Controls:**
- **Left Click** — Spawn a single particle
- **Right Click** — Find nearest particle + spawn a random cluster
- **Middle Click** — Randomize color

**UI Features:**
- "Switch Colliders" button to toggle between circle and rectangle colliders
- Separate sliders for circle radius, rect width, and rect height
- Gravity and particle size sliders

---

### 4. Simple N-Body Scene (O(n²))

A gravitational simulation using direct pairwise force calculation between every particle. Suitable for small particle counts.

**Controls:**
- **Left Click** — Spawn a single particle with configurable mass
- **Right Click** — Spawn a galaxy cluster (100 particles with tangential orbital velocities)
- **Middle Click + Drag** — Pan the camera
- **Scroll Wheel** — Zoom in/out towards the mouse position

**UI Features:**
- Particle size, particle mass, and particle radius sliders
- Live FPS and particle count display

**Algorithm:** Every particle exerts gravitational force on every other particle each frame. Complexity: **O(n²)**.

---

### 5. Barnes-Hut Algorithm N-Body Scene (O(n log n))

An optimized gravitational simulation using the Barnes-Hut quadtree algorithm. Handles thousands of particles efficiently.

**Controls:**
- **Left Click** — Spawn a single particle with configurable mass
- **Right Click** — Mode dependent (see UI radio buttons):
  - *Spawn a small Galaxy:* 100 particles with orbital velocities around the cursor
  - *Spawn heavy Particle:* A single heavy particle at the cursor
  - *Spawn a Galaxy with 3000 Particles:* A pre-configured galaxy with a central heavy mass and orbiting particles
- **Middle Click + Drag** — Pan the camera
- **Scroll Wheel** — Zoom in/out towards the mouse position

**UI Features:**
- Particle size and mass sliders (affect newly spawned particles)
- Gravity slider
- Right click mode selection (Small Galaxy / Heavy Particle / Full Galaxy)
- Collision toggle (warning: performance-intensive with many particles)
- Live FPS and particle count display

**Algorithm:** Each frame, a quadtree is built from all particle positions. For each quadrant, the total mass and center of mass are computed. When calculating gravitational force on a particle, distant quadrants (where `quadrant_size² < distance² × θ²`) are approximated as a single body. This reduces complexity from **O(n²) to O(n log n)**. The quadtree implementation is based on [DeadlockCode/barnes-hut](https://github.com/DeadlockCode/barnes-hut) and adapted for our Verlet-based physics.

---

### 6. Lava Lamp Scene

A lava lamp simulation using oscillating magnets to push and pull particles inside a rectangular container.

**Controls:**
- **Left Click** — Creates a blast effect that pushes nearby particles away from the cursor

**UI Features:**
- Particle size and magnet strength sliders
- Particle count slider (applied on reset/start)
- Lamp size scaling slider
- "Reset" and "Start" buttons to reinitialize the simulation

---

## Usage of AI/LLMs
- **Gemini** 
  - Learning how to use Git
  - Refactoring code
  - Generating Unit Tests
  - Learning how to create a physics engine 
  - General Questions about Rust and different libraries
  - Suggestions for dependencies
- **Claude** 
  - Debugging (Quadtree, Gravity, Camera, egui Event-Handling)
  - Understanding WGPU and shader pipeline
  - Help with complex version-specific errors
  - Help to implement complex functions
  - Explanation of concepts (Verlet integration, Barnes-Hut algorithm)
  - Help to understand new libraries 
- **ChatGPT** 
  - Find & fix bugs during development 
  - Understand the principles behind the physics
  - Refactoring code & help with complex fuctions
  - Creating a Background Image
  
## Testing

All tests can be run with:

```bash
cargo test --workspace
```

### Why Unit Tests?

Unit tests are particularly valuable for a physics simulation like this, because bugs are often hard to identify visually. A particle that collides slightly wrong or a camera that drifts by a few pixels may look fine at first glance but leads to increasingly incorrect behavior over time. With unit tests we can catch these problems early and reliably.

## Quality Gates

The repository has the following server-side quality gates configured for pull requests:
- [Gate 1 description, e.g. "CI must pass (cargo build + cargo test)"]
- [Gate 2 description, e.g. "At least two approving review required"]
