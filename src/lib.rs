pub mod board;

use std::fmt;

use board::{Board, Cell, Player, at_pos, HEIGHT, WIDTH};
use anyhow::{Result, anyhow};

/*
    A game struct representing the current Reversi game state.
*/
#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    board: board::Board,
    current_player: Player,
}

impl Game {
    // TODO: this will be horrendously inefficient, however, i want to get test cases in place first,
    // so i'm doing rudimentary solutions for me to work out later
    fn moves(&self) -> Vec<usize> {
        let mut moves = Vec::new();

        // loop through all cells and check if they are valid moves
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                if self.is_valid_move(x, y) {
                    moves.push(at_pos(x, y));
                }
            }
        }

        moves
    }
    fn is_valid_move(&self, x_init: usize, y_init: usize) -> bool {
        let cell = self.board.get_cell(x_init, y_init);

        if cell != Cell::Empty {
            return false;
        }

        let opposing_tile = Cell::Player(self.current_player.opponent());

        let mut tiles_to_flip: Vec<usize> = Vec::new();

        let directions: &[(isize, isize)] = &[
            (0, 1),
            (1, 1),
            (1, 0),
            (1, -1),
            (0, -1),
            (-1, -1),
            (-1, 0),
            (-1, 1),
        ];

        for (x_dir, y_dir) in directions {
            let mut x = x_init;
            let mut y = y_init;

            x = x.wrapping_add_signed(*x_dir);
            y = y.wrapping_add_signed(*y_dir);

            if !self.board.on_board(x, y) || self.board.get_cell(x, y) != opposing_tile {
                continue;
            }

            x = x.wrapping_add_signed(*x_dir);
            y = y.wrapping_add_signed(*y_dir);

            if !self.board.on_board(x, y) {
                continue;
            }

            while self.board.get_cell(x, y) == opposing_tile {
                x = x.wrapping_add_signed(*x_dir);
                y = y.wrapping_add_signed(*y_dir);

                if !self.board.on_board(x, y) {
                    break;
                }
            }

            if !self.board.on_board(x, y) {
                continue;
            }

            if self.board.get_cell(x, y) == Cell::Player(self.current_player) {
                loop {
                    x = x.checked_add_signed(-*x_dir).unwrap();
                    y = y.checked_add_signed(-*y_dir).unwrap();

                    if x == x_init && y == y_init {
                        break;
                    }

                    tiles_to_flip.push(at_pos(x, y));
                }
            }
        }
        
        tiles_to_flip.len() > 0
    }

    pub fn new() -> Game {
        let mut board = Board::new();

        board.set_cell(3, 3, Cell::Player(Player::One));
        board.set_cell(4, 4, Cell::Player(Player::One));

        board.set_cell(3, 4, Cell::Player(Player::Two));
        board.set_cell(4, 3, Cell::Player(Player::Two));

        Game {
            board,
            current_player: Player::One,
        }
    }

    pub fn play(&mut self, x: usize, y: usize) -> Result<()> {
        let moves = self.moves();

        if !moves.contains(&at_pos(x, y)) {
            return Err(anyhow!("Invalid move"));
        }

        self.board.set_cell(x, y, Cell::Player(self.current_player));

        self.current_player = self.current_player.opponent();
        Ok(())
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Current player: {}", Cell::Player(self.current_player).to_char())?;

        let moves = self.moves();

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let character = if moves.contains(&at_pos(x, y)) {
                    '*'
                } else {
                    self.board.get_cell(x, y).to_char()
                };

                write!(f, "{}", character)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}