use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Player {
    One,
    Two,
}

impl Player {
    pub fn opponent(&self) -> Self {
        match self {
            Player::One => Player::Two,
            Player::Two => Player::One
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cell {
    Empty,
    Player(Player),
}

pub const WIDTH: usize = 8;
pub const HEIGHT: usize = 8;
pub const SIZE: usize = WIDTH * HEIGHT;

impl Cell {
    pub fn to_char(&self) -> char {
        match self {
            Cell::Empty => '-',
            Cell::Player(Player::One) => 'X',
            Cell::Player(Player::Two) => 'O',
        }
    }
}

/// Represents the internal state of the game board.
#[derive(Clone, Debug, PartialEq)]
pub struct Board {
    cells: [Cell; SIZE],
}

pub fn at_pos(x: usize, y: usize) -> usize {
    x + y * WIDTH
}

impl Board {
    pub fn new() -> Board {
        Board {
            cells: [Cell::Empty; SIZE],
        }
    }

    pub fn total_moves(&self) -> usize {
        self.cells.iter().filter(|cell| **cell != Cell::Empty).count()
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Cell {
        self.cells[at_pos(x, y)]
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: Cell) {
        self.cells[at_pos(x, y)] = cell;
    }

    pub fn set_cell_idx(&mut self, idx: usize, cell: Cell) {
        self.cells[idx] = cell;
    }

    pub fn get_cell_idx(&self, idx: usize) -> Cell {
        self.cells[idx]
    }

    pub fn on_board(&self, x: usize, y: usize) -> bool {
        x < WIDTH && y < HEIGHT
    }
}

impl IntoIterator for Board {
    type Item = Cell;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.to_vec().into_iter()
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                write!(f, "{}", self.get_cell(x, y).to_char())?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
