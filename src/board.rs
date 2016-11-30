use std::ops::{Index, IndexMut};

#[derive(Copy, Clone, PartialEq)]
pub enum Player { X, O }
pub type Cell = Option<Player>;
#[derive(Copy, Clone)]
pub struct Board(pub [[Cell; 3]; 3]);

impl Board {
    pub fn new() -> Self {
        Board([[None; 3]; 3])
    }
}

impl Index<usize> for Board {
    type Output = [Cell; 3];

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Board {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut [Cell; 3] {
        &mut self.0[index]
    }
}
