use crate::{Game, board::SIZE};
use anyhow::Result;

pub fn negamax(game: &Game) -> Result<isize> {
    let moves = &game.moves();

    if moves.is_empty() {
        return Ok(0);
    }

    for possible_move in moves {
        if game.is_winning_move_idx(*possible_move, game.current_player)? {
            return Ok((SIZE as isize + 1 - game.total_moves() as isize) / 2);
        }
    }

    let mut best_score = -(SIZE as isize);

    for possible_move in moves {
        let mut new_game = game.clone();

        new_game.play_idx(*possible_move)?;
    
        let score = -negamax(&new_game)?;

        if score > best_score {
            best_score = score;
        }
    }

    Ok(best_score)
}

/// Returns the scores for each move
pub fn solve(game: &Game) -> Vec<(isize, usize)> {
    // turn each possible move into a score and get the top ones
    game.moves().into_iter().map(|possible_move| {
        let mut new_game = game.clone();
        new_game.play_idx(possible_move).unwrap();
        let score = negamax(&new_game).unwrap();
        (score, possible_move)
    }).collect()
}