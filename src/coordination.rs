//! Contains the setup method for the [GameCoordinator] struct, which is meant to modify the [PlayField] state, receive & handle player input, set things up, enforce the play phases etc.
//! This module holds the game loop & some auxiliary helper functions.

use muehle::game::{painting::*, Field, PlayerColor};

use muehle::game::state::PlayField;

use smallvec::SmallVec;
use smartstring::alias::CompactString;

mod game_phases;
mod setup;

#[derive(Clone, Copy)]
pub enum GamePhase {
    Start,
    Set,
    MoveAndJump,
    Terminated,
}

pub struct GameCoordinator {
    play_field: PlayField,
    // 0 = Player 1, 1 = Player 2
    player_names: (CompactString, CompactString),
    round: u32,
    game_phase: GamePhase,
    // false -> Player 1, true -> Player 2
    turn: bool,
    error_state: bool,
}

impl GameCoordinator {
    // TODO Refactor in game-loop.rs
    pub fn start_game(&mut self) {
        let mut player_won = false;

        while let GamePhase::Start = self.game_phase {
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

            self.game_phase = GamePhase::Set;
        }
        println!("\n> Starting with {}!", EMP.paint("Set-Phase"));

        let mut changes_to_highlight = SmallVec::<[Field; 3]>::new();
        let mut set_rounds_done = 0;

        while set_rounds_done < 9 {
            let (player_color, player_name) =
                self.print_turn_header(self.game_phase, Some(set_rounds_done), &changes_to_highlight);

            changes_to_highlight.clear();
            let input_field = self.get_field_coord_input("> Enter a field a stone should be placed on: ");
            changes_to_highlight.push(input_field);

            match self.play_field.try_set(input_field, player_color) {
                Ok(_) => println!(
                    "> Successfully placed {} on {} for {}.",
                    HIGHLIGHT.paint(player_color),
                    HIGHLIGHT.paint(format!("{}{}", input_field.0, input_field.1)),
                    EMP.paint(player_name)
                ),
                Err(err) => {
                    print_error(&format!("{}", err));

                    self.error_state = true;
                    continue;
                }
            }

            // If a mill ocurred & a stone was stolen, print info message & set game states according to the
            // left amount of stones on the field. Only the opponents amount of stones changes
            if let Some(mut mills) = self.do_mills_interaction(input_field, player_color) {
                changes_to_highlight.append(&mut mills);
                //for mill in mills { TODO ?
                //if changes_to_highlight.contains(&mill) {
                //}
                //}
            };

            self.error_state = false;
            self.round += 1;
            self.turn = !self.turn;

            if let PlayerColor::Black = player_color {
                set_rounds_done += 1;
            }
        }

        self.game_phase = GamePhase::MoveAndJump;
        println!("\n> Starting with {}!", EMP.paint("Move-Phase"));

        while let GamePhase::MoveAndJump = self.game_phase {
            let (player_color, player_name) = self.print_turn_header(self.game_phase, None, &changes_to_highlight);

            changes_to_highlight.clear();
            let start_field = self.get_field_coord_input("> Enter the stone you want to move: ");
            changes_to_highlight.push(start_field);
            let target_field = self.get_field_coord_input("> Enter it's target position: ");
            changes_to_highlight.push(target_field);

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

                    self.error_state = true;
                    continue;
                }
            }

            if let Some(mut mills) = self.do_mills_interaction(target_field, player_color) {
                changes_to_highlight.append(&mut mills);
            }

            // The opponent of the current play might have lost a stone:
            let player_and_amount_of_stones = match player_color {
                PlayerColor::White => (&self.player_names.1, self.play_field.amount_of_stones.1),
                PlayerColor::Black => (&self.player_names.0, self.play_field.amount_of_stones.0),
            };

            // One player has less than 2 stones and has lost the game. Mutates self.phase
            if player_and_amount_of_stones.1 <= 2 {
                println!(
                    ">\n> {} only has {} stones left. Terminating game.\n>",
                    EMP.paint(player_and_amount_of_stones.0),
                    HIGHLIGHT.paint(player_and_amount_of_stones.1)
                );

                player_won = player_and_amount_of_stones.0 != &self.player_names.0;
                self.game_phase = GamePhase::Terminated;
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

            self.error_state = false;
            self.round += 1;
            self.turn = !self.turn;
        }

        assert!(matches!(self.game_phase, GamePhase::Terminated));

        let winners_name = match player_won {
            true => &self.player_names.0,
            false => &self.player_names.1,
        };
        println!(
            "> {}",
            EMP.paint(format!("{} won the match! Congratulations!", winners_name))
        );

        // TODO Ask for another round
    }
}

impl GameCoordinator {
    /// Returns the (real, \in [0,1]) player number which currently is on turn
    /// Turn is initially set to the player who choose the white color.
    fn which_players_turn(&self) -> u32 {
        (self.turn as u32) + 1
    }

    /// Returns a tuple which is used at the beginning of each round to display the current players name & the round no
    fn get_current_turns_attributes(&self) -> (&str, PlayerColor) {
        match self.which_players_turn() {
            1 => (self.player_names.0.as_str(), self.get_player_color()),
            2 => (self.player_names.1.as_str(), self.get_player_color()),
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

    /// Wrapper for [print_plain] method of [PlayField], adding line breaks around it's output.
    ///
    fn print_play_field(&self) {
        println!("\n");
        self.play_field.print_plain();
        println!("\n");
    }

    /// Just another wrapper, but able to highlight the game field on specified points - by using the ^2 rt complexity :(
    /// If you ask me, it's worth it
    fn print_play_field_highlighted(&self, to_highlight: &[Field]) {
        println!("\n");
        self.play_field.print_and_highlight(to_highlight);
        println!("\n");
    }
}

/// Shorthand for equal error printing
fn print_error(message: &str) {
    println!("> {}\n <", ERROR.paint(message))
}
