#[cfg(test)]
mod tests {
    use reversi_solver::Game;

    #[test]
    fn it_works() {
        let game = Game::new();

        println!("test");

        println!("{}", game);
    }
}