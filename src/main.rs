use clap::{Parser, Subcommand};
use reversi_solver::{Game, solve::negamax};
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
        step: bool,
    },
    /// Solve a game
    Solve
}


fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Random { step } => {
            let mut game = Game::new();

            let mut moves = game.moves();

            while moves.len() > 0 {
                let move_index = fastrand::usize(..moves.len());
                let chosen_move = moves[move_index];

                if step {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    print!("{}[2J", 27 as char);
                    println!("{}", game);
                }

                game.play_idx(chosen_move).unwrap();

                moves = game.moves();
                if moves.len() == 0 {
                    game.swap_players();
                    moves = game.moves();
                }
            }
            

            println!("{}", game);
        },
        Commands::Solve => {
            println!("{}", negamax(&Game::new())?);
        }
    };

    Ok(())
}