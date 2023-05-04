use muehle_game::game::logic::GameCoordinator;

fn main() {
    let mut coordinator = GameCoordinator::setup();
    coordinator.start_game();
}
