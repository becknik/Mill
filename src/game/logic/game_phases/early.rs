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
                    let input_buffer = String::from(input_buffer.trim());

                    if input_buffer.is_empty() {
                        print_error("Please enter a name which actually holds some characters.");
                        continue;
                    }

                    if current_player_assigned_to == 1 {
                        player1 = Some(input_buffer);
                        current_player_assigned_to += 1;

                        println!("> Here we go, {}!", EMP.paint(player1.clone().unwrap()));
                    } else {
                        if player1.clone().unwrap() == input_buffer {
                            print_error("Player are the same.");
                            continue;
                        }
                        player2 = Some(input_buffer);

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
            players: (
                smartstring::alias::CompactString::from(player1.unwrap()),
                smartstring::alias::CompactString::from(player2.unwrap()),
            ),
            phase: GamePhase::Start,
            turn: false,
            round: 0,
        }
    }

    pub fn setup_player_colors(&self) -> bool {
        let playing_white_id = loop {
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
                    if input_buffer == self.players.0 {
                        break false;
                    } else if input_buffer == self.players.1 {
                        break true;
                    } else if let Ok(int) = input_buffer.parse::<i32>() {
                        if !(1..3).contains(&int) {
                            print_error("Your input must be either 1 or 2 or a players name. Please try again.");
                        } else {
                            break int != 1;
                        }
                    } else {
                        print_error("Your input must be either 1 or 2 or a players name. Please try again.");
                    }
                }
                Err(error) => print_error(&format!("> Error processing input: {}\n", error)),
            }
        };
        playing_white_id
    }
}
