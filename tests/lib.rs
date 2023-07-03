#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use reversi_solver::Game;

    #[test]
    fn test_games() {
        let mut games_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        games_path.push("tests/resources/games.txt");

        let raw_games = std::fs::read_to_string(games_path).unwrap();
        
        let game_lines =  raw_games.lines().collect::<Vec<_>>();
        let games = game_lines.chunks(9);

        for game in games {
            assert!(game.len() == 9);

            let should_fail = match game[0] {
                "nofail" => false,
                "fail" => true,
                _ => panic!("Invalid test case {}", game[0]),
            };
            
            let parsed_game = Game::from_string(&game[1..].join("\n"), true);

            if should_fail {
                assert!(parsed_game.is_err(), "Game should have failed to parse: {game:?}");
            } else {
                parsed_game.unwrap();
            }
        }
    }
}