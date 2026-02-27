use glam::Vec2;
use std::collections::HashMap;

// for position based fluid implementation after Miles Macklin and Matthias Mueller

#[derive(Default)]
pub struct NeighborGrid {
    pub cell_size: f32,
    buckets: HashMap<(i32, i32), Vec<usize>>,
}

impl NeighborGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size: cell_size.max(1e-6),
            buckets: HashMap::new(),
        }
    }

    pub fn set_cell_size(&mut self, cell_size: f32) {
        self.cell_size = cell_size.max(1e-6);
    }

    pub fn cell_of(&self, p: Vec2) -> (i32, i32) {
        let cx = (p.x / self.cell_size).floor() as i32;
        let cy = (p.y / self.cell_size).floor() as i32;
        (cx, cy)
    }

    pub fn clear(&mut self) {
        self.buckets.clear();
    }

    // build grid from only particle position without recognizing the entire particle
    pub fn rebuild_from_particles<P>(
        &mut self,
        particles: &[P],
        mut get_pos: impl FnMut(&P) -> Vec2,
    ) {
        self.clear();
        for (i, p) in particles.iter().enumerate() {
            let pos = get_pos(p);
            let cell = self.cell_of(pos);
            self.buckets.entry(cell).or_default().push(i);
        }
    }

    //search neighbbour in radius
    pub fn for_each_neighbor_in_radius<F>(&self, positions: &[Vec2], i: usize, h: f32, mut f: F)
    where
        F: FnMut(usize),
    {
        let h2 = h * h;
        let xi = positions[i];
        let (cx, cy) = self.cell_of(xi);

        for oy in -1..=1 {
            for ox in -1..=1 {
                let cell = (cx + ox, cy + oy);
                let Some(list) = self.buckets.get(&cell) else {
                    continue;
                };

                for &j in list {
                    // distance test
                    if j == i {
                        continue;
                    }
                    let r = positions[j] - xi;
                    if r.length_squared() < h2 {
                        f(j);
                    }
                }
            }
        }
    }
}

// unit test generated with ChatGPT
#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn finds_neighbors_within_radius() {
        let mut grid = NeighborGrid::new(1.0);

        // 3 particles
        let positions = vec![
            Vec2::new(0.0, 0.0), // 0
            Vec2::new(0.5, 0.0), // 1 (close)
            Vec2::new(2.0, 0.0), // 2 (further away)
        ];

        // build grid
        grid.rebuild_from_particles(&positions, |p| *p);

        // capture neighbours of particle 0
        let mut neighbors = Vec::new();
        grid.for_each_neighbor_in_radius(&positions, 0, 1.0, |j| {
            neighbors.push(j);
        });

        neighbors.sort();
        assert_eq!(neighbors, vec![1]);
    }
    #[test]
    fn finds_neighbors_across_cell_boundary() {
        let mut grid = NeighborGrid::new(1.0);

        let positions = vec![
            Vec2::new(0.99, 0.0), // Grid (0,0)
            Vec2::new(1.01, 0.0), // Grid (1,0)
        ];

        grid.rebuild_from_particles(&positions, |p| *p);

        let mut neighbors = Vec::new();
        grid.for_each_neighbor_in_radius(&positions, 0, 0.1, |j| {
            neighbors.push(j);
        });

        assert_eq!(neighbors, vec![1]);
    }
    #[test]
    fn neighbor_relation_is_symmetric() {
        let mut grid = NeighborGrid::new(1.0);

        let positions = vec![Vec2::new(0.0, 0.0), Vec2::new(0.4, 0.0)];

        grid.rebuild_from_particles(&positions, |p| *p);

        let mut a = Vec::new();
        let mut b = Vec::new();

        grid.for_each_neighbor_in_radius(&positions, 0, 1.0, |j| a.push(j));
        grid.for_each_neighbor_in_radius(&positions, 1, 1.0, |j| b.push(j));

        assert_eq!(a, vec![1]);
        assert_eq!(b, vec![0]);
    }
}
