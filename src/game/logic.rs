//! Contains the setup method for the [GameCoordinator] struct, which is meant to modify the [PlayField] state, receive & handle player input, set things up, enforce the play phases etc.
//! This module holds the game loop & some auxiliary helper functions.

pub mod constants {
    use once_cell::sync::Lazy;
    use yansi::Style;

    const EMP_COLOR: (u8, u8, u8) = (193, 49, 0);
    pub static EMP: Lazy<Style> = Lazy::new(|| Style::new(yansi::Color::RGB(EMP_COLOR.0, EMP_COLOR.1, EMP_COLOR.2)));

    pub static HIGHLIGHT: Lazy<Style> = Lazy::new(|| Style::new(yansi::Color::Blue));
    pub static ERROR: Lazy<Style> = Lazy::new(|| Style::new(yansi::Color::Red).bold());
}

mod game_phases;

use self::constants::*;

use self::game_phases::*;

use super::{
    state::{representation::types::Field, PlayField},
    PlayerColor,
};
use smallvec::SmallVec;
use smartstring::alias::CompactString;

pub struct GameCoordinator {
    play_field: PlayField,
    // 0 = Player 1, 1 = Player 2
    players: (CompactString, CompactString),
    phase: GamePhase,
    // false -> Player 1, true -> Player 2
    turn: bool,
    round: u32,
}

impl GameCoordinator {
    // TODO Refactor in game-loop.rs
    pub fn start_game(&mut self) {
        let mut last_changes = SmallVec::<[Field; 5]>::new();

        let mut error_occurred = false;
        let mut player_won = false;

        loop {
            match self.phase {
                GamePhase::Start => {
                    println!("> Starting the game!");
                    let playing_white_id = self.setup_player_colors();

                    // White begins: if player id is 2, set turn to 1 for player 2 to start
                    if playing_white_id {
                        self.turn = true;
                    }
                    self.round = 1;

                    println!(
                        "> {} plays {}.",
                        EMP.paint(format!("Player {}", self.which_players_turn())),
                        HIGHLIGHT.paint("white")
                    );

                    self.phase = GamePhase::Set;

                    println!("\n> Starting with {}!", EMP.paint("Set-Phase"));
                }
                GamePhase::Set => {
                    let mut rounds_done = (0, 0);
                    while rounds_done.0 < 9 || rounds_done.1 < 9 {
                        let (player_color, player_name) =
                            self.print_turn_header(self.phase, Some(rounds_done.1), &last_changes, error_occurred);

                        let input_field = self.get_field_coord_input("> Enter a field a stone should be placed on: ");
                        last_changes.clear();
                        last_changes.push(input_field);

                        match self.play_field.try_set(input_field, player_color) {
                            Ok(_) => println!(
                                "> Successfully placed {} on {} for {}.",
                                HIGHLIGHT.paint(player_color),
                                HIGHLIGHT.paint(format!("{}{}", input_field.0, input_field.1)),
                                EMP.paint(player_name)
                            ),
                            Err(err) => {
                                print_error(&format!("{}", err));
                                error_occurred = true;
                                continue;
                            }
                        }

                        if let Some(mills) = self.mills_interaction(input_field, player_color) {
                            for mill in mills {
                                if last_changes.contains(&mill) {
                                    last_changes.push(mill);
                                }
                            }
                        };

                        error_occurred = false;
                        self.round += 1;
                        self.turn = !self.turn;

                        match player_color {
                            PlayerColor::White => rounds_done.0 += 1,
                            PlayerColor::Black => rounds_done.1 += 1,
                        }
                    }

                    self.phase = GamePhase::MoveAndJump;

                    println!("\n> Starting with {}!", EMP.paint("Move-Phase"));
                }
                GamePhase::MoveAndJump => {
                    let (player_color, player_name) =
                        self.print_turn_header(self.phase, None, &last_changes, error_occurred);

                    let start_field = self.get_field_coord_input("> Enter the stone you want to move: ");
                    let target_field = self.get_field_coord_input("> Enter it's target position: ");
                    last_changes.clear();
                    last_changes.push(start_field);
                    last_changes.push(target_field);

                    // Print out the coords if move was successful, else continue loop
                    match self.play_field.try_move(start_field, target_field, player_color) {
                        Ok(_) => println!(
                            "> {} successfully moved a {} stone from {} to {}.",
                            EMP.paint(player_name),
                            HIGHLIGHT.paint(player_color),
                            HIGHLIGHT.paint(format!("{}{}", start_field.0, start_field.1)),
                            HIGHLIGHT.paint(format!("{}{}", target_field.0, target_field.1))
                        ),
                        Err(err) => {
                            print_error(&format!("{}", err));
                            error_occurred = true;
                            continue;
                        }
                    }

                    // If a mill ocurred & a stone was stolen, print info message & set game states according to the
                    // left amount of stones on the field. Only the opponents amount of stones changes
                    if let Some(mills) = self.mills_interaction(target_field, player_color) {
                        for mill in mills {
                            if last_changes.contains(&mill) {
                                last_changes.push(mill);
                            }
                        }

                        let player_and_amount_of_stones = match player_color {
                            PlayerColor::White => (&self.players.1, self.play_field.amount_of_stones.1),
                            PlayerColor::Black => (&self.players.0, self.play_field.amount_of_stones.0),
                        };

                        // One player has less than 2 stones and has lost the game. Mutates self.phase
                        if player_and_amount_of_stones.1 <= 2 {
                            println!(
                                ">\n> {} only has {} stones left. Terminating game.\n>",
                                EMP.paint(player_and_amount_of_stones.0),
                                HIGHLIGHT.paint(player_and_amount_of_stones.1)
                            );

                            player_won = player_and_amount_of_stones.0 != &self.players.0;
                            self.phase = GamePhase::Terminated;
                        // Info message, allowing jumps for player with only 3 stones left
                        } else if player_and_amount_of_stones.1 == 3 {
                            println!(
                                ">\n> {} only has {} stones left. Starting with {}!\n>",
                                EMP.paint(player_and_amount_of_stones.0),
                                HIGHLIGHT.paint(player_and_amount_of_stones.1),
                                EMP.paint("Jump-Phase")
                            );
                        // Normal info message printing out new amount of stones on the playfield
                        } else {
                            println!(
                                ">\n> {} only has {} stones left.\n>",
                                EMP.paint(player_and_amount_of_stones.0),
                                HIGHLIGHT.paint(player_and_amount_of_stones.1),
                            );
                        }
                    }

                    error_occurred = false;
                    self.round += 1;
                    self.turn = !self.turn;
                }
                GamePhase::Terminated => {
                    let player_name = match player_won {
                        true => &self.players.0,
                        false => &self.players.1,
                    };
                    println!(
                        "> {}",
                        EMP.paint(format!("{} won the match! Congratulations!", player_name))
                    );
                    // TODO Ask for another round
                    break;
                }
            }
        }
    }
}

impl GameCoordinator {
    /// Returns the player number which currently is on turn
    /// Turn is initially set to the player who choose the white color.
    fn which_players_turn(&self) -> u32 {
        (self.turn as u32) + 1
    }

    /// Returns a tuple which is used at the beginning of each round to display the current players name & the round no
    fn get_current_turns_attributes(&self) -> (&str, PlayerColor) {
        match self.which_players_turn() {
            1 => (self.players.0.as_str(), self.get_player_color()),
            2 => (self.players.1.as_str(), self.get_player_color()),
            _ => panic!(),
        }
    }

    /// Returns the player color of the player currently being on turn
    fn get_player_color(&self) -> PlayerColor {
        let current_round = self.round % 2;

        match current_round {
            1 => PlayerColor::White,
            0 => PlayerColor::Black,
            _ => panic!(),
        }
    }

    /// Wrapper for [print_plain] method of [PlayField], adding line breaks around it's output
    fn print_play_field(&self) {
        print!("\n\n");
        self.play_field.print_plain();
        print!("\n\n");
    }

    /// Just another wrapper
    fn print_play_field_highlighted(&self, to_highlight: &[Field]) {
        print!("\n\n");
        self.play_field.print_and_highlight(to_highlight);
        print!("\n\n");
    }
}

/// Shorthand for equal error printing
fn print_error(message: &str) {
    println!("> {}\n", ERROR.paint(message))
}
