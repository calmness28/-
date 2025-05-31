use macroquad::prelude::*;

pub struct Vec2Pool {
    pool: Vec<Vec2>,
    used_count: usize,
}

impl Vec2Pool {
    pub fn new(capacity: usize) -> Self {
        Self {
            pool: vec![Vec2::ZERO; capacity],
            used_count: 0,
        }
    }

    pub fn get(&mut self, x: f32, y: f32) -> &mut Vec2 {
        if self.used_count >= self.pool.len() {
            self.pool.push(Vec2::new(x, y));
        } else {
            self.pool[self.used_count] = Vec2::new(x, y);
        }
        let result = &mut self.pool[self.used_count];
        self.used_count += 1;
        result
    }

    pub fn reset(&mut self) {
        self.used_count = 0;
    }

    pub fn get_slice(&self) -> &[Vec2] {
        &self.pool[..self.used_count]
    }
}