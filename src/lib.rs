use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cell {
    Empty,
    Player1,
    Player2,
}

const WIDTH: usize = 8;
const HEIGHT: usize = 8;
const SIZE: usize = WIDTH * HEIGHT;

impl Cell {
    fn to_char(&self) -> char {
        match self {
            Cell::Empty => ' ',
            Cell::Player1 => 'X',
            Cell::Player2 => 'O',
        }
    }
}

/*
    Represents the internal state of the game board.
*/
pub struct Board {
    cells: [Cell; SIZE],
}

impl Board {
    pub fn new() -> Board {
        Board {
            cells: [Cell::Empty; SIZE],
        }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Cell {
        self.cells[y * WIDTH + x]
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: Cell) {
        self.cells[y * WIDTH + x] = cell;
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
