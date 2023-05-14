use coordination::GameCoordinator;

mod coordination;

fn main() {
    let mut coordinator = GameCoordinator::setup();
    coordinator.start_game();
}
