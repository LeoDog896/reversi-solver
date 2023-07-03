use owo_colors::OwoColorize;
use rand::Rng;
use regex::Regex;
use std::cmp::Ordering;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io;
use std::io::stdout;
use std::io::Write;

// HashMap is used in order to assign a count to each element inside win/draw/loss stats
use std::collections::HashMap;
use std::iter;

// IndexSet provides an indexed HashSet to allow returning element by index
// Used for getting random items from set in O(1) time so MCTS is more efficient
// Docs: https://docs.rs/indexmap/1.5.0/indexmap/set/struct.IndexSet.html
use indexmap::IndexSet;

// Used to limit MCTS duration
use std::time::{Duration, Instant};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug)]
enum Square {
    Empty,
    Player,
    Cpu,
}

/**
 * Game Board Struct
 *
 * Manages the board vector and the information about it, including...
 *      - perimeter tiles
 *      - whether it is the players turn
 *      - available actions for both player and cpu
*/
#[derive(Clone)]
struct Board {
    width: u8,
    board_size: u8,
    board: Vec<Square>,
    perimeter: IndexSet<u8>,
    player_available_actions: IndexSet<u8>,
    cpu_available_actions: IndexSet<u8>,
    player_turn: bool,
}

/**
 * Board object functions
 */
impl Board {
    /**
     * Initializes a Reversi game board
     *
     */
    fn new(width: u8, height: u8) -> Self {
        let size = width * height;
        let mut player_actions: IndexSet<u8> = IndexSet::new();
        let mut cpu_actions: IndexSet<u8> = IndexSet::new();
        let mut perimeter_tiles: IndexSet<u8> = IndexSet::new();
        let mut new_board = vec![Square::Empty; (size).into()];

        new_board[28] = Square::Player;
        new_board[35] = Square::Player;
        new_board[27] = Square::Cpu;
        new_board[36] = Square::Cpu;

        player_actions.insert(26);
        player_actions.insert(19);
        player_actions.insert(37);
        player_actions.insert(44);

        cpu_actions.insert(29);
        cpu_actions.insert(20);
        cpu_actions.insert(34);
        cpu_actions.insert(43);

        perimeter_tiles.insert(18);
        perimeter_tiles.insert(19);
        perimeter_tiles.insert(20);
        perimeter_tiles.insert(21);
        perimeter_tiles.insert(26);
        perimeter_tiles.insert(29);
        perimeter_tiles.insert(34);
        perimeter_tiles.insert(37);
        perimeter_tiles.insert(42);
        perimeter_tiles.insert(43);
        perimeter_tiles.insert(44);
        perimeter_tiles.insert(45);

        Self {
            width,
            board_size: size,
            board: new_board,
            perimeter: perimeter_tiles,
            player_available_actions: player_actions,
            cpu_available_actions: cpu_actions,
            player_turn: true, // Player always takes the first turn
        }
    }

    /**
     * Handles a piece being put onto the board
     *
     * Adds to board -> flips pieces -> update perimeter -> updates available actions -> change turns
     */
    fn ins(&mut self, pos: u8, val: Square) {
        // Add new tile to board
        let pos_u: usize = if self.get_available_actions().contains(&pos) {
            pos.into()
        } else {
            println!("ERROR: {} is not a valid action", pos);
            return;
        };

        self.board.splice(pos_u..=pos_u, iter::once(val));

        let mut tiles = Vec::new();

        // Manages the direction of iteration
        for direction in 0..8 {
            // This part of the function iterates in all 8 directions from the tile, checking if any of
            // the tiles in these directions will be flipped -> that is, they are...
            //                      - adjacent to the newly placed tile, or
            //                      - in a span of opposing tiles adjacent to the newly placed tile, and
            //                      - has a tile on the other side of the opposing tiles that "sandwiches"
            //                          them with no empty spaces inbetween

            let mut u: u8 = 1;
            tiles.clear();

            loop {
                // Depending on direction, changes the formula for iteration
                let new_pos: u8 = match get_new_pos(direction, pos, u, self.board_size) {
                    None => break,
                    Some(x) => Some(x).unwrap(),
                };

                let new_pos_usize: usize = new_pos.into();

                let tile = self.board.get(new_pos_usize).unwrap();

                // Refer to comment above for explanation
                if tile != &val && tile != &Square::Empty {
                    tiles.push(new_pos);
                } else if tile == &val {
                    for t in &tiles {
                        self.add(*t, val);
                    }
                } else {
                    tiles.clear();
                    break;
                }

                u += 1;
            }
        }

        // Remove inserted tile from perimeter
        self.perimeter.remove(&pos);

        // Adds the specified spaces to perimeter IndexSet
        // Update perimeter above
        for i in 0..3 {
            let new_pos: u8 = match pos.checked_sub(9 - i) {
                None => continue,
                Some(x) => Some(x).unwrap(),
            };
            let new_pos_usize: usize = new_pos.into();
            if self.board.get(new_pos_usize).unwrap() == &Square::Empty {
                // implement row overflow handling
                self.perimeter.insert(new_pos);
            }
        }

        if let Some(x) = pos.checked_sub(1) {
            if self.board.get::<usize>(x.into()).unwrap() == &Square::Empty {
                self.perimeter.insert(x);
            }
        }

        // Update perimeter to the right
        match pos + 1 < self.board_size {
            true => {
                let new_pos = pos + 1;
                let new_pos_usize: usize = new_pos.into();
                if self.board.get(new_pos_usize).unwrap() == &Square::Empty {
                    self.perimeter.insert(new_pos);
                }
            }
            false => (),
        }

        // Update perimeter below
        for i in 0..3 {
            let new_pos: u8 = pos + 9 - i;
            let new_pos_usize: usize = new_pos.into();
            if new_pos < self.board_size && self.board.get(new_pos_usize).unwrap() == &Square::Empty
            {
                self.perimeter.insert(new_pos);
            }
        }

        // Update available actions
        self.player_available_actions.remove(&pos);
        self.cpu_available_actions.remove(&pos);

        // For each player Player and CPU
        for player in &[Square::Player, Square::Cpu] {
            // For each tile in the perimeter
            for tile in self.get_perimeter() {
                // Check if that tile is an available action
                self.check_tile_actions(tile, *player);
            }
        }

        // Alternate turns
        if self.player_turn {
            self.player_turn = false;
        } else {
            self.player_turn = true;
        }
    }

    /**
     * Given a tile position it will check in all directions if it is an available option
     * for player with the input val (1 or 2)
     */
    fn check_tile_actions(&mut self, pos: u8, val: Square) {
        let mut tiles = Vec::new();

        // Manages the direction of iteration
        for direction in 0..8 {
            let mut u: u8 = 1;
            tiles.clear();

            loop {
                // Depending on direction, changes the formula for iteration
                let new_pos: u8 = match get_new_pos(direction, pos, u, self.board_size) {
                    None => break,
                    Some(x) => Some(x).unwrap(),
                };

                let new_pos_usize: usize = new_pos.into();
                let tile = self.board.get(new_pos_usize).unwrap(); // Gets value from tile at new position

                if tile != &val && tile != &Square::Empty {
                    // If the tile is not the same color as inserted, add to tiles vec
                    tiles.push(new_pos);
                } else if tile == &val && !tiles.is_empty() {
                    // If there is a tile the same color as the initial val with opposing tiles inbetween...
                    if val == Square::Player {
                        self.player_available_actions.insert(pos);
                    } else {
                        self.cpu_available_actions.insert(pos);
                    }
                    tiles.clear();
                    return;
                } else {
                    // Else, blank tile means not available action
                    if val == Square::Player {
                        self.player_available_actions.remove(&pos);
                    } else {
                        self.cpu_available_actions.remove(&pos);
                    }

                    tiles.clear();
                    break;
                }
                u += 1;
            }
        }
    }

    /**
     * Returns a clone of the IndexSet of available actions depending on which players turn it is
     *
     * Should only use this function to get the available actions, don't individually
     * reference the player or cpu sets
     */
    fn get_available_actions(&self) -> IndexSet<u8> {
        if self.player_turn {
            self.get_player_actions()
        } else {
            self.get_cpu_actions()
        }
    }

    fn get_player_actions(&self) -> IndexSet<u8> {
        IndexSet::clone(&self.player_available_actions)
    }

    fn get_cpu_actions(&self) -> IndexSet<u8> {
        IndexSet::clone(&self.cpu_available_actions)
    }

    const fn is_player_turn(&self) -> bool {
        self.player_turn
    }

    /**
     * Returns IndexSet of the tiles in the perimeter of the board pieces
     */
    fn get_perimeter(&self) -> IndexSet<u8> {
        IndexSet::clone(&self.perimeter)
    }

    // Returns:
    // 0 -> incomplete
    // 1 -> player win
    // 2 -> cpu win
    // 3 -> draw
    fn check_game_state(&self) -> u8 {
        let player_actions = self.get_player_actions();
        let cpu_actions = self.get_cpu_actions();

        // GAME IS ENDED
        if cpu_actions.is_empty() || player_actions.is_empty() {
            let (player_score, cpu_score): (u8, u8) = self.get_score();

            match player_score.cmp(&cpu_score) {
                Ordering::Greater => 1,
                Ordering::Less => 2,
                Ordering::Equal => 3,
            }
        } else {
            0
        }
    }

    /**
     * get_score() -> returns tuple containing current score for player and cpu
     */
    fn get_score(&self) -> (u8, u8) {
        let mut count_player = 0;
        let mut count_cpu = 0;

        for i in 0..64 {
            match self.board.get(i).unwrap() {
                Square::Empty => continue,
                Square::Player => count_player += 1,
                Square::Cpu => count_cpu += 1,
            }
        }

        (count_player, count_cpu)
    }

    /**
     * Add value at position on board
     *
     * val = 0: unused square
     * val = 1: player piece
     * val = 2: cpu piece
     */
    fn add(&mut self, pos: u8, val: Square) {
        let pos_u: usize = pos.into();
        self.board.splice(pos_u..(pos_u + 1), iter::once(val));
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (player_score, cpu_score): (u8, u8) = self.get_score();

        writeln!(f, "\n     {}", "A B C D E F G H".bold())?;

        for (count, i) in self.board.iter().enumerate() {
            let count = count as u8;
            if count % self.width == 0 {
                if count != 0 {
                    let row_num: u8 = count / 8;
                    write!(f, "{}\n     ", row_num.to_string().bold())?;
                } else {
                    write!(f, "     ")?;
                }
            }
            if i == &Square::Player {
                write!(f, "{} ", "●".red())?;
            } else if i == &Square::Cpu {
                write!(f, "{} ", "●".green())?;
            } else if self.player_available_actions.contains(&count) {
                write!(f, "{} ", "*".bold())?;
            } else {
                write!(f, "- ")?;
            }
        }
        write!(f, "{}\n\n", "8".bold())?;

        writeln!(
            f,
            "     Player: {}, CPU: {}\n",
            player_score.to_string().red(),
            cpu_score.to_string().green()
        )?;

        Ok(())
    }
}

/**
 * Returns a new position based on direction, initial pos, iteration, and board size
 * Intended to be used in a loop (such as in the Board.ins() function)
 *
 * @returns: Some(x) if new position is on board, or
 * @returns: None if position overflows board
 */
fn get_new_pos(dir: u8, pos: u8, iter: u8, size: u8) -> Option<u8> {
    let new_pos: Option<u8> = match dir {
        0 => {
            // Right
            let position = pos + iter;
            if position % 8 == 0 {
                None
            } else {
                Some(position)
            }
        }

        1 => {
            // Left
            pos.checked_sub(iter).filter(|&x| Some(x).unwrap() % 8 != 7)
        }

        2 => {
            // Down
            let position = pos + (iter * 8);
            if position < size {
                Some(position)
            } else {
                None
            }
        }

        3 => {
            // Up
            pos.checked_sub(iter * 8)
        }

        4 => {
            // Up left: must check that doesn't % 8 = 7 and doesn't overflow
            pos.checked_sub(iter * 8 + iter).filter(|&x| x % 8 != 7)
        }

        5 => {
            // Up right: must check that doesn't % 8 = 0 and doesn't overflow
            pos.checked_sub(iter * 8 - iter).filter(|&x| x % 8 != 0)
        }

        6 => {
            // Down left: must check that doesnt % 8 = 7 and
            let position = pos + (iter * 8) - iter;
            if position < size && position % 8 != 7 {
                Some(position)
            } else {
                None
            }
        }

        7 => {
            // Down left: must check that doesnt % 8 = 7 and
            let position = pos + (iter * 8) + iter;
            if position < size && position % 8 != 0 {
                Some(position)
            } else {
                None
            }
        }

        _ => None,
    };

    new_pos
}

/**
 * Convert 2d string index to vector index
 * @params:     s: &str - len 2 string of char A-H followed by int 1-8
 * @returns:    u8 position in 1d Vec
 */
fn convert_2d(s: &str) -> u8 {
    // Handle panic
    let letter = s.chars().next().unwrap().to_ascii_lowercase();
    let num = s.chars().nth(1).unwrap();

    let col: u8 = match letter {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => 42,
    };

    // Probably better way to do this.... but I couldn't find it
    let row: u8 = match num {
        '1' => 0,
        '2' => 1,
        '3' => 2,
        '4' => 3,
        '5' => 4,
        '6' => 5,
        '7' => 6,
        '8' => 7,
        _ => 42,
    };

    row * 8 + col
}

/**
 * Convert integer vector index into 2d string index
 * Note: this function is the inverse of `convert_2d`()
 * @params:     num: less than 64 valued integer representing 1d index of vector
 * @returns:    String of values [a-h][1-8]
 */
fn convert_num(num: u8) -> String {
    let val: f64 = (num / 8).into();

    let letter: &str = match val.floor() as u8 {
        0 => "A",
        1 => "B",
        2 => "C",
        3 => "D",
        4 => "E",
        5 => "F",
        6 => "G",
        7 => "H",
        _ => {
            println!("ERROR convert_num() -> input too large");
            "ERR"
        }
    };

    format!("{}{}", letter, num % 8 + 1)
}

fn print_title() {
    println!("################################################################");
    println!("#                                                              #");
    println!(
        "#                {}                #",
        "Welcome to Reversi against AI!".bold()
    );
    println!("#                                                              #");
    println!("################################################################\n\n");
}

fn print_help() {
    println!("\nCommands:\n");
    println!(
        "  {}  -  print the current available actions",
        "actions".bold()
    );
    println!("  {}  -  show game rules", "rules".bold());
    println!("  {}     -  quit the game", "exit".bold());
    println!();
}

fn print_actions(actions: IndexSet<u8>) {
    print!("\nPlayer's Actions: ");
    for action in actions {
        print!("{} ", convert_num(action).bold());
    }
    println!("\n");
}

fn print_rules() {
    println!(
        "      #                {}                #\n",
        "REVERSI RULES".bold()
    );
    println!(
        " * {} tiles represent the user's spots, {} represent the CPUs.\n",
        "Red".red(),
        "Green".green()
    );
    println!(" * The user starts by placing a tile adjacent to a green tile.\n Possible actions are marked by asterisks (*) on the board.\n");
    println!(" * The game ends when either player cannot play a piece or the\n board is full.  The player with the most tiles wins.\n");
}

/**
 * Simplified Monte Carlo Tree Search which performs random playouts until completion
 * and records the win/draw/loss statistics for each available action at current board state.
 *  Parameters:
 *      `b`              -    the current board state to initialize the playout board
 *      `max_steps`      -    maximum number of iterations
 *      `timer`          -    maximum amount of time to spend during the mcts in seconds
 *
 */
fn monte_carlo_tree_search(b: &Board, max_steps: usize, timer: usize, diff: &String) -> u8 {
    let mut stats: [Vec<u8>; 3] = [vec![], vec![], vec![]];
    let start_time = Instant::now();

    for i in 0..max_steps {
        print!(".");
        stdout().flush().unwrap();

        if (i + 1) % 30 == 0 {
            println!();
        }

        // Break out of function when timer is reached
        if start_time.elapsed() >= Duration::new(timer as u64, 0) {
            break;
        }

        let actions = b.get_available_actions();

        for action in actions {
            let mut playout_board: Board = b.clone();

            match random_playout(&mut playout_board, diff) {
                1 => stats[1].push(action), // 1 -> Player wins so add action to loss list
                2 => stats[0].push(action), // 2 -> CPU wins so add action to win list
                3 => stats[2].push(action), // 3 -> Game draw so add action to draw list
                _ => continue,
            };
        }
    }

    // Populate hashmap with frequency of elements in win list
    let mut a = HashMap::new();
    for i in &stats[0] {
        if a.contains_key(i) {
            *(a.get_mut(&i).unwrap()) += 1;
        } else {
            a.insert(i, 1);
        }
    }

    // Returns the highest value in frequency hashmap as best play if win list exists,
    // else return a random action if no elements exist in win list.
    if stats[0].is_empty() {
        let actions = b.get_available_actions();
        let actions_size = actions.len();
        let rand_index = rand::thread_rng().gen_range(0..actions_size);
        let rand_val = actions.get_index(rand_index).unwrap();
        *rand_val
    } else {
        **a.iter()
            .max_by(|a, b| a.1.cmp(b.1))
            .map(|(k, _v)| k)
            .unwrap()
    }
}

/**
*   Performs random playouts or uses a heuristic to perform the next move based on the diff parameter.
        - if diff is set to easy, then the playouts will be random actions
        - if diff is set to hard, playouts will use the Max Tile Heuristic
*/
fn random_playout(b: &mut Board, diff: &String) -> u8 {
    // Play a game until completion
    loop {
        match b.check_game_state() {
            0 => {
                // Game not done
                if !b.player_turn {
                    let actions = b.get_cpu_actions();
                    let actions_size = actions.len();

                    match diff.as_str() {
                        // EASY
                        "1" => {
                            let rand_index = rand::thread_rng().gen_range(0..actions_size);
                            let rand_val = actions.get_index(rand_index).unwrap();
                            b.ins(*rand_val, Square::Cpu);
                        }

                        // HARD
                        "2" => {
                            let new_val = get_max_tile(b);
                            if new_val == 99 {
                                continue;
                            } // Someone ran out of moves
                            b.ins(new_val, Square::Cpu);
                        }
                        _ => println!(
                            "ERROR in random_playout() -> diff variable invalid: {}",
                            diff
                        ),
                    };
                } else {
                    let actions = b.get_player_actions();
                    let actions_size = actions.len();
                    let rand_index = rand::thread_rng().gen_range(0..actions_size);
                    let rand_val = actions.get_index(rand_index).unwrap();
                    b.ins(*rand_val, Square::Player);
                }

                continue;
            }
            1 => return 1, // Player Wins
            2 => return 2, // CPU Wins
            3 => return 3, // Draw
            _ => return 42,
        };
    }
}

/**
 * Max Tile Heuristic
 *      - Returns the position that results in the highest score out of all possible actions
 *      - If no actions are available, then return an error code of 99 to indicate game end                      
 */
fn get_max_tile(b: &Board) -> u8 {
    let actions = b.get_available_actions();
    let (_, prev_cpu_score): (u8, u8) = b.get_score();
    let best_score = prev_cpu_score;
    let mut best_pos: u8 = 0;

    if actions.is_empty() {
        return 99;
    }

    for action in actions {
        // check increase in value of tiles
        let mut new_board: Board = b.clone();

        new_board.ins(action, Square::Cpu);

        let (_, cpu_score): (u8, u8) = new_board.get_score();

        if cpu_score > best_score {
            best_pos = action;
        }
    }

    best_pos
}

fn main() {
    const MAX_STEPS: usize = 1000;
    const TIME: usize = 5;
    const WIDTH: u8 = 8;
    const HEIGHT: u8 = 8;

    print_title();
    print_rules();

    let mut cpu_diff = String::new();

    // Get difficulty
    let difficulty: String = loop {
        println!("\n[1] Easy");
        println!("[2] Hard\n");
        println!("Select CPU Difficulty (1, 2): ");
        io::stdin()
            .read_line(&mut cpu_diff)
            .expect("Failed to read line");

        match cpu_diff.trim().to_string().as_str() {
            "1" | "2" => {
                break cpu_diff.trim().to_string();
            }
            _ => {
                println!("ERROR: Invalid entry");
                cpu_diff = String::new();
                continue;
            }
        }
    };

    let mut board = Board::new(WIDTH, HEIGHT);
    let re = Regex::new(r"([aA-hH][1-8])").unwrap();

    // =============
    // Player VS CPU
    // =============
    loop {
        match board.check_game_state() {
            1 => {
                println!("Player has won");
                println!("{board}");
                break;
            }
            2 => {
                println!("CPU has won");
                println!("{board}");
                break;
            }
            3 => {
                println!("Game is a draw");
                println!("{board}");
                break;
            }
            _ => (),
        };

        println!("{board}");

        if board.is_player_turn() {
            println!("Place piece at position: ");
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            // Validate input string
            if re.is_match(&input) {
                let input_u8: u8 = convert_2d(&input);
                board.ins(input_u8, Square::Player);
            } else {
                match input.as_str() {
                    "help\n" => {
                        print_help();
                        continue;
                    }
                    "actions\n" => {
                        print_actions(board.get_player_actions());
                        continue;
                    }
                    "rules\n" => {
                        print_rules();
                        continue;
                    }
                    "exit\n" => break,
                    _ => {
                        println!("ERROR: invalid input, enter 'help' for command information");
                        continue;
                    }
                };
            };
        } else {
            let best_play: u8 = monte_carlo_tree_search(&board, MAX_STEPS, TIME, &difficulty);
            println!("\n\nCPU found {} as best play", convert_num(best_play));
            board.ins(best_play, Square::Cpu);
        }
    }
}
