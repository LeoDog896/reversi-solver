pub mod board;
pub mod solve;

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
    pub fn moves(&self) -> Vec<usize> {
        let mut moves = Vec::new();

        // loop through all cells and check if they are valid moves
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                if let Some(_) = self.is_valid_move(x, y) {
                    moves.push(at_pos(x, y));
                }
            }
        }

        moves
    }

    pub fn swap_players(&mut self) {
        self.current_player = self.current_player.opponent();
    }

    fn winning_player(&self) -> Option<Player> {
        let mut player_one_count = 0;
        let mut player_two_count = 0;

        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                match self.board.get_cell(x, y) {
                    Cell::Player(Player::One) => player_one_count += 1,
                    Cell::Player(Player::Two) => player_two_count += 1,
                    _ => (),
                }
            }
        }

        if player_one_count > player_two_count {
            Some(Player::One)
        } else if player_two_count > player_one_count {
            Some(Player::Two)
        } else {
            None
        }
    }

    pub fn is_winning_move(&self, x: usize, y: usize, player: Player) -> Result<bool> {
        let mut new_game = self.clone();

        new_game.play(x, y)?;

        Ok(new_game.winning_player() == Some(player) && new_game.moves().is_empty())
    }

    pub fn is_winning_move_idx(&self, index: usize, player: Player) -> Result<bool> {
        let (x, y) = (index % WIDTH, index / WIDTH);

        self.is_winning_move(x, y, player)
    }

    fn is_valid_move(&self, x_init: usize, y_init: usize) -> Option<Vec<usize>> {
        let cell = self.board.get_cell(x_init, y_init);

        if cell != Cell::Empty {
            return None;
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
        
        if tiles_to_flip.is_empty() {
            None
        } else {
            Some(tiles_to_flip)
        }
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

    pub fn play_idx(&mut self, index: usize) -> Result<()> {
        let move_set = self.is_valid_move(index % WIDTH, index / WIDTH).ok_or(anyhow!("Invalid move"))?;

        self.board.set_cell_idx(index, Cell::Player(self.current_player));

        for idx in move_set {
            self.board.set_cell_idx(idx, Cell::Player(self.current_player));
        }

        self.current_player = self.current_player.opponent();
        Ok(())
    }

    pub fn play(&mut self, x: usize, y: usize) -> Result<()> {
        self.play_idx(at_pos(x, y))
    }
    
    pub fn from_string(string: &str, player: Player, validate: bool) -> Result<Self> {
        let mut game = Self::new();

        let mut recorded_possible_moves: Vec<usize> = Vec::new();

        let rows = string.split('\n');

        for (y, row) in rows.enumerate() {
            if y >= HEIGHT {
                Err(anyhow!("Too many rows"))?;
            }

            for (x, character) in row.chars().enumerate() {
                if x >= WIDTH {
                    Err(anyhow!("Too many columns"))?;
                }

                let cell = match character {
                    'X' => Cell::Player(Player::One),
                    'O' => Cell::Player(Player::Two),
                    '*' => {
                        recorded_possible_moves.push(at_pos(x, y));
                        continue;
                    }
                    '-' => Cell::Empty,
                    _ => Err(anyhow!("Invalid character: {}", character))?,
                };

                game.board.set_cell(x, y, cell);
            }
        }

        game.current_player = player;

        if validate {
            let mut moves = game.moves();

            moves.sort_unstable();
            recorded_possible_moves.sort_unstable();

            if moves != recorded_possible_moves {
                Err(anyhow!("real != recorded moves: \n{:?} != {:?}\nparsed game:\n{game}", moves, recorded_possible_moves))?;
            }
        }

        Ok(game)
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