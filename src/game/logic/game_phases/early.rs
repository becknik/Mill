use std::io::{self, Write};

use crate::game::{
    logic::{print_error, GameCoordinator, EMP, HIGHLIGHT},
    state::PlayField,
};

use super::GamePhase;

impl GameCoordinator {
    pub fn setup() -> Self {
        let mut player1: Option<String> = None;
        let player2: Option<String>;
        let mut current_player_assigned_to = 1;

        loop {
            print!(
                "> Ok {}, please enter your username: ",
                EMP.paint(format!("Player {}", current_player_assigned_to))
            );
            io::stdout().flush().unwrap();

            let mut input_buffer = String::new();
            match io::stdin().read_line(&mut input_buffer) {
                Ok(_) => {
                    let input_buffer = input_buffer.trim();

                    if input_buffer.is_empty() {
                        print_error("Please enter a name which actually holds some characters.");
                        continue;
                    }

                    if player1 == None {
                        player1 = Some(input_buffer.to_string());
                        current_player_assigned_to += 1;

                        // .clone().unwrap() looks like bad library design for me...
                        println!("> Here we go, {}!", EMP.paint(player1.clone().unwrap()));
                    } else {
                        if player1.clone().unwrap() == input_buffer {
                            print_error("Player are the same.");
                            continue;
                        }
                        player2 = Some(input_buffer.to_string());

                        println!("> Here we go, {}!", EMP.paint(player2.clone().unwrap()));
                        break;
                    }
                }
                Err(e) => print_error(&format!("Error evaluating your input: {}", e)),
            }
        }
        println!();

        GameCoordinator {
            play_field: PlayField::new(),
            player_names: (
                smartstring::alias::CompactString::from(player1.unwrap()),
                smartstring::alias::CompactString::from(player2.unwrap()),
            ),
            round: 0,
            game_phase: GamePhase::Start,
            turn: false,
            error_state: false,
        }
    }

    /// Returns the player playing white, coded as 0 for player 1, 1 for player 2
    pub fn setup_player_colors(&self) -> bool {
        let error_message = "Input must either be 1, 2 or a players name. Please try again.";

        return loop {
            println!(
                "> Which player wants to play with the {} >>{}<<?",
                HIGHLIGHT.paint("white stones"),
                HIGHLIGHT.paint(crate::game::PlayerColor::White)
            );
            print!(
                "> Please enter a {} or the {}: ",
                EMP.paint("players name"),
                EMP.paint("player's number")
            );
            io::stdout().flush().unwrap();

            let mut input_buffer = String::new();
            match io::stdin().read_line(&mut input_buffer) {
                Ok(_) => {
                    let input_buffer = input_buffer.trim();
                    // Player 0/ 1 shall play white
                    if input_buffer == self.player_names.0 {
                        break false;
                    // Player 1/ 2 shall play white
                    } else if input_buffer == self.player_names.1 {
                        break true;
                    } else if let Ok(int) = input_buffer.parse::<i32>() {

                        if !(1..3).contains(&int) {
                            print_error(error_message);
                        } else {
                            break int != 1;
                        }
                    } else {
                        print_error(error_message);
                    }
                }
                Err(error) => print_error(&format!("> Error processing input: {}\n", error)),
            }
        };
    }
}
