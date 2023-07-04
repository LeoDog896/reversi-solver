use clap::{Parser, Subcommand};
use reversi_solver::{Game, solve::negamax, board::Player};
use anyhow::Result;

/// Solve and generate reversi puzzles
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Makes a random game
    Random {
        #[arg(short, long, default_value_t = false)]
        slow: bool,
        
        #[arg(short, long, default_value_t = 0)]
        backtrack: usize
    },
    /// Solve a game
    Solve
}


fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Random { slow, backtrack } => {
            let mut game = Game::new();

            let mut decided_moves: Vec<usize> = Vec::new();

            let mut moves = game.moves();

            while moves.len() > 0 {
                let move_index = fastrand::usize(..moves.len());
                let chosen_move = moves[move_index];

                if slow {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    print!("{}[2J", 27 as char);
                    println!("{}", game);
                }

                game.play_idx(chosen_move).unwrap();

                decided_moves.push(chosen_move);

                moves = game.moves();
                if moves.len() == 0 {
                    game.swap_players();
                    moves = game.moves();
                }
            }

            let mut final_game = Game::new();
            
            for decided_move in &decided_moves[0..decided_moves.len() - backtrack] {
                final_game.play_idx(*decided_move).unwrap();
            }
            

            println!("{}", final_game);
        },
        Commands::Solve => {
            let game = Game::from_string("--OOOOOO\n\
            -**OOXXO\n\
            *-OOOOOO\n\
            XO*OXOOO\n\
            XOOOXOOO\n\
            XOXOXOOO\n\
            XOOXXOOO\n\
            *OXXXXO*", Player::One, true)?;
            println!("{}", negamax(&game)?);
        }
    };

    Ok(())
}