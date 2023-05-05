//! Contains the actual game logic which is meant to modify the [PlayField] state, handle player input, set things up, etc.
const EMP_COLOR: (u8, u8, u8) = (193, 49, 0);

enum GamePhase {
    Start,
    Set,
    Move,
    Jump,
    Terminated,
}

use std::{
    fmt::format,
    io::{self, Read, Write},
    num,
};

use super::{state::PlayField, PlayerColor};
use smartstring::alias::CompactString;
use yansi::Paint;

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
            let player_fmted = Paint::rgb(
                EMP_COLOR.0,
                EMP_COLOR.1,
                EMP_COLOR.2,
                format!("Player {}", current_player_assigned_to),
            );
            print!("> Ok {}, please enter your username: ", player_fmted);
            io::stdout().flush().unwrap();

            let mut input_buffer = String::new();
            match io::stdin().read_line(&mut input_buffer) {
                Ok(_) => {
                    let input_buffer = String::from(input_buffer.trim());

                    println!(
                        "> Here we go, {}!",
                        Paint::rgb(EMP_COLOR.0, EMP_COLOR.1, EMP_COLOR.2, input_buffer.clone())
                    );

                    // TODO: Perform checks on names

                    if current_player_assigned_to == 1 {
                        player1 = Some(input_buffer);
                        current_player_assigned_to += 1
                    } else {
                        player2 = Some(input_buffer);
                        break;
                    }
                }
                Err(e) => println!("Error evaluating your input: {}", Paint::red(e.to_string())),
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
        loop {
            println!();

            match self.phase {
                GamePhase::Start => {
                    println!("> Starting the game!");
                    let playing_white_id = loop {
                        println!(
                            "> Which player wants to play with the {} ⚪:?",
                            Paint::blue("white stones")
                        );
                        print!(
                            "> Please enter a {} or the {}: ",
                            Paint::rgb(EMP_COLOR.0, EMP_COLOR.1, EMP_COLOR.2, "players name"),
                            Paint::rgb(EMP_COLOR.0, EMP_COLOR.1, EMP_COLOR.2, "player's number")
                        );
                        io::stdout().flush().unwrap();

                        let mut input_buffer = String::new();
                        match io::stdin().read_line(&mut input_buffer) {
                            Ok(_) => {
                                let input_buffer = input_buffer.trim();
                                if input_buffer == self.players.0 {
                                    break 0;
                                } else if input_buffer == self.players.1 {
                                    break 1;
                                } else if let Ok(int) = input_buffer.parse::<i32>() {
                                    if int < 1 || 3 <= int {
                                        println!(
                                            "{}\n",
                                            Paint::red(
                                                "Your input must be either 1 or 2 or a players name. Please try again."
                                            )
                                        )
                                    } else {
                                        break int;
                                    }
                                } else {
                                    println!(
                                        "{}\n",
                                        Paint::red(
                                            "Your input must be either 1 or 2 or a players name. Please try again."
                                        )
                                    )
                                }
                            }
                            Err(error) => println!("Error processing input: {}", Paint::red(error)),
                        }
                    };

                    // White begins
                    if  playing_white_id == 1 {
                         self.turn = true;
                    }
                    self.round = 1;

                    println!(
                        "> {} plays white",
                        Paint::rgb(
                            EMP_COLOR.0,
                            EMP_COLOR.1,
                            EMP_COLOR.2,
                            format!(
                                "Player {}", self.which_players_turn_is_it() as i32
                            )
                        )
                    );
                }
                GamePhase::Set => todo!(),
                GamePhase::Move => todo!(),
                GamePhase::Jump => todo!(),
                GamePhase::Terminated => todo!(),
            }
        }
    }
}

impl GameCoordinator {
    // TODO Refactor in output.rs
    fn print_play_field(&self) {
        self.play_field.print();
    }

    /// Returns the player number coded as bool, where false stands for player 0 & true for player 1.
    /// Due to white always starting to set stones, white should start
    fn which_players_turn_is_it(&self) -> bool {
        self.round % 2 == 1
    }
}
