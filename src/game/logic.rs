//! Contains the actual game logic which is meant to modify the [PlayField] state, handle player input, set things up, etc.
enum GamePhase {
    Start,
    Set,
    Move,
    Jump,
    Terminated,
}

use std::io;

use super::state::PlayField;
use smartstring::alias::CompactString;

pub struct GameCoordinator {
    play_field: PlayField,
    players: (CompactString, CompactString),
    phase: GamePhase,
    turn: bool,
    round: u32,
}

impl GameCoordinator {
    pub fn setup() -> Self {
        let mut player1: Option<String> = None;
        let mut player2: Option<String> = None;
        let mut current_player_assigned_to = 1;

        loop {
            println!("Player {current_player_assigned_to}, please enter your username:");

            let mut input_buffer = String::new();
            match io::stdin().read_line(&mut input_buffer) {
                Ok(_) => {
                    println!("Here we go, {input_buffer}");

                    // TODO: Perform checks on names

                    if current_player_assigned_to == 1 {
                        player1 = Some(input_buffer);
                        current_player_assigned_to += 1
                    } else {
                        player2 = Some(input_buffer);
                        break;
                    }
                }
                Err(e) => println!("Error evaluating your input: {}\n Please try again.\n", e.to_string()),
            }
        }

        GameCoordinator {
            play_field: PlayField::new(),
            players: (
                CompactString::from(player1.unwrap()),
                CompactString::from(player2.unwrap()),
            ),
            phase: GamePhase::Start,
            turn: false,
            round: 0,
        }
    }

	// TODO Refactor in game-loop.rs
    pub fn start_game(&mut self) {
        self.print_play_field();
        //while !matches!(self.phase, GamePhase::Terminated) {
        //match self.phase {
        //GamePhase::Start => todo!(),
        //GamePhase::Set => todo!(),
        //GamePhase::Move => todo!(),
        //GamePhase::Jump => todo!(),
        //GamePhase::Terminated => todo!(),
        //}
        //}
    }
}

impl GameCoordinator {

	// TODO Refactor in output.rs
    fn print_play_field(&self) {
        self.play_field.print();
    }
}
