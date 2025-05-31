use std::collections::{HashMap, HashSet};
use crate::position::Position;

pub struct SpatialHash {
    cell_size: i32,
    grid: HashMap<(i32, i32), HashSet<Position>>,
}

impl SpatialHash {
    pub fn new(cell_size: i32) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.grid.clear();
    }

    pub fn insert(&mut self, pos: Position) {
        let cell = (pos.x / self.cell_size, pos.y / self.cell_size);
        self.grid.entry(cell).or_insert_with(HashSet::new).insert(pos);
    }

    pub fn contains(&self, pos: Position) -> bool {
        let cell = (pos.x / self.cell_size, pos.y / self.cell_size);
        if let Some(positions) = self.grid.get(&cell) {
            positions.contains(&pos)
        } else {
            false
        }
    }
}