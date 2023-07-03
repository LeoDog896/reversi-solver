#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use reversi_solver::{Game, board::Player};

    #[test]
    fn test_games() {
        let mut games_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        games_path.push("tests/resources/games.txt");

        let raw_games = std::fs::read_to_string(games_path).unwrap();
        
        let game_lines =  raw_games.lines().collect::<Vec<_>>();
        let games = game_lines.chunks(9);

        for game in games {
            assert!(game.len() == 9);

            let header = game[0].split(" ").collect::<Vec<_>>();
            assert!(header.len() == 2);

            let should_fail = match header[0] {
                "nofail" => false,
                "fail" => true,
                _ => panic!("Invalid test case {}", header[0]),
            };

            let player = match header[1] {
                "X" => Player::One,
                "O" => Player::Two,
                _ => panic!("Invalid player {}", header[1]),
            };
            
            let parsed_game = Game::from_string(&game[1..].join("\n"), player, true);

            if should_fail {
                assert!(parsed_game.is_err(), "Game should have failed to parse: {game:?}");
            } else {
                parsed_game.unwrap();
            }
        }
    }
}