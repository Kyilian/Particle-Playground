use glam::Vec2;
use std::collections::HashMap;


#[derive(Default)]
pub struct NeighborGrid {
    pub cell_size: f32,
    buckets: HashMap<(i32, i32), Vec<usize>>,
}


impl NeighborGrid{
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size: cell_size.max(1e-6),
            buckets: HashMap::new(),
        }
    }

    pub fn set_cell_size(&mut self, cell_size: f32) {
        self.cell_size = cell_size.max(1e-6);
    }

    pub fn cell_of(&self, p:Vec2) -> (i32,i32){
        let cx = (p.x / self.cell_size);
        let cy = (p.y / self.cell_size);
        (cx,cy)
    }

    pub fn clear(&mut self) {
        self.buckets.clear();
    }

    pub fn 

}