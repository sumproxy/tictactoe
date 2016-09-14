use std::ops::{Index, IndexMut};

#[derive(Copy, Clone, PartialEq)]
pub enum Player { X, O }
pub type Cell = Option<Player>;
pub struct Board([[Cell; 3]; 3]);

impl Board {
    pub fn new() -> Self {
        Board([[None; 3]; 3])
    }

    pub fn winner(&self, last_move: (usize, usize)) -> Option<Player> {
        let (y, x) = last_move;
        let board = self.0;
        let player = board[y][x];

        // check row
        if board[y][0] == player && board[y][1] == player && board[y][2] == player {
            return player;
        }

        // check column
        if board[0][x] == player && board[1][x] == player && board[2][x] == player {
            return player;
        }

        match (y, x) {
            // not on a diagonal
            (0, 1) | (1, 0) | (1, 2) | (2, 1) => None,
            
            (y, x) => {
                // on a primary diagonal
                if y == x && board[0][0] == player && board[1][1] == player && board[2][2] == player {
                    player
                }
                // on a secondary diagonal
                else if y + x == 2 && board[0][2] == player && board[1][1] == player && board[2][0] == player {
                    player
                }
                // everything else
                else {
                    None
                }
            }
        }
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

impl<'a> IntoIterator for &'a Board {
    type Item = &'a [Cell; 3];
    type IntoIter = ::std::slice::Iter<'a, [Cell; 3]>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
