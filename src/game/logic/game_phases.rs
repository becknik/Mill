use std::io::{self, Write};

use smallvec::SmallVec;
use smartstring::alias::CompactString;

use crate::game::{
    logic::{
        constants::{EMP, HIGHLIGHT},
        print_error,
    },
    state::representation::{constants::FIELD_LUT, types::Field},
};

mod early;

#[derive(Clone, Copy)]
pub enum GamePhase {
    Start,
    Set,
    MoveAndJump,
    Terminated,
}

impl super::GameCoordinator {
    /// Returns valid coordinates of the game field in A_G, 1-7 mapping.
    /// Loops & requests input until the provided input is valid. Handles all error cases.
    pub fn get_field_coord_input(&self, message: &str) -> Field {
        let vaild_input = loop {
            print!("{}", message);
            io::stdout().flush().unwrap();

            let mut input_buffer = String::new();
            match io::stdin().read_line(&mut input_buffer) {
                Ok(_) => {
                    let input_buffer = input_buffer.trim();

                    if input_buffer.len() < 2 {
                        print_error("Provided input is to short.");
                        continue;
                    }

                    let row = match input_buffer[0..1].parse::<char>() {
                        Ok(c) if c.is_alphabetic() => c.to_uppercase().next().unwrap(),
                        Ok(_) => {
                            print_error("Provided input character is not alphabetic.");
                            continue;
                        }
                        Err(_) => {
                            print_error("Input does't start with a letter representing a column.");
                            continue;
                        }
                    };
                    let column = match input_buffer[1..2].parse::<u8>() {
                        Ok(n) => n,
                        Err(_) => {
                            print_error("Input is ill formatted. Second letter should be a number representing a row.");
                            continue;
                        }
                    };

                    break (row, column);
                }
                Err(error) => print_error(&format!("Error processing input: {error}",)),
            }
        };
        vaild_input
    }

    /// Returns if mills were detected & returns them if so
    /// Internally prints them out
    pub fn check_for_and_get_mils(&self, last_updated_field: Field) -> Option<SmallVec<[Field; 5]>> {
        let mills = self.play_field.get_mill_crossing(last_updated_field);

        // This hurts. And I'm not sure how to do better.
        if mills.len() == 3 {
            let field_1 = mills[0];
            let field_2 = mills[1];
            let field_3 = mills[2];
            print!(
                "\n> Detected a mill for fields: {}!",
                EMP.paint(format!(
                    "({}{}, {}{}, {}{})",
                    field_1.0, field_1.1, field_2.0, field_2.1, field_3.0, field_3.1,
                ))
            );

            Some(mills)
        } else if mills.len() == 6 {
            let field_1 = mills[0];
            let field_2 = mills[1];
            let field_3 = mills[2];
            let field_4 = mills[3];
            let field_5 = mills[4];
            let field_6 = mills[5];
            print!(
                "\n> Detected {} mills on {} and {}!!\n> Your opponent must be sleeping, be a 3 year old, or you must be testing extreme cases ;)",
                EMP.paint("TWO"),
                EMP.paint(format!(
                    "({}{}, {}{}, {}{})",
                    field_1.0, field_1.1, field_2.0, field_2.1, field_3.0, field_3.1,
                )),
                EMP.paint(format!(
                    "({}{}, {}{}, {}{})",
                    field_4.0, field_4.1, field_5.0, field_5.1, field_6.0, field_6.1,
                ))
            );

            Some(mills)
        } else {
            None
        }
    }

    /// Handles the mill cross-check of the last field a stone was set upon.
    /// Includes the user interaction part for selecting a valid field on the [PlayField].
    /// Handled extreme cases:
    /// - All stones on the play field are element of mills
    /// TODO This is to weak. If the player e.g. has 3 stones & all are in a mill, it must be skipped too...
    ///
    /// Returns true if a mill was detected for the [GamePhase] cases to trigger coordinative behavior.
    pub fn mills_interaction(
        &mut self,
        input_field: (char, u8),
        player_color: crate::game::PlayerColor,
    ) -> Option<SmallVec<[Field; 5]>> {
        if let Some(mills) = self.check_for_and_get_mils(input_field) {
            self.print_play_field_highlighted(&mills);
            let mut amount_of_mills = mills.len() / 3;

            let mut stones_in_mills = 0;
            for coord in FIELD_LUT {
                // Every mill should be detected exactly 3 times
                stones_in_mills += self.play_field.get_mill_crossing(coord).len() / 3;
            }

            let (white_stones, black_stones) = self.play_field.amount_of_stones;
            let all_stones_in_mills = stones_in_mills as u32 == (white_stones + black_stones);

            // While here are mill on the last set position left & not all stones are element of a mill: Prompt to take stones
            while 0 < amount_of_mills && !all_stones_in_mills {
                let field_to_take = self.get_field_coord_input("> Enter the stone do you want to take: ");

                match self.play_field.try_take(field_to_take, player_color) {
                    Ok(_) => println!(
                        "> Successfully took stone on {}",
                        EMP.paint(format!("{}{}", field_to_take.0, field_to_take.1))
                    ),
                    Err(message) => {
                        print_error(message);
                        continue;
                    }
                }

                amount_of_mills -= 1;
            }

            if all_stones_in_mills {
                println!("> Detected as many stones on the play field as mills. There is nothing to take.");
            }

            Some(mills)
        } else {
            None
        }
    }

    /// Prints out the current round, the state of the play field and messages for some phases of [GamePhase].
    /// Also skips this print outs, if the provided [error_occurred] is true.
    /// Returns some convenient values needed in the game phases for coordination of the [PlayField].
    pub fn print_turn_header(
        &self,
        phase: GamePhase,
        black_rounds_done: Option<u32>,
        highlight: &[Field],
        error_occurred: bool,
    ) -> (crate::game::PlayerColor, smartstring::SmartString<smartstring::Compact>) {
        let (player_name, player_color) = self.get_current_turns_attributes();
        let player_name = CompactString::from(player_name);

        // Print out the round and game field info, if no error occurred
        if !error_occurred {
            print!("\n\n\t\t  ===============\n");
            print!("\t\t  === {} ===\n", HIGHLIGHT.paint(format!("Round {}", self.round)));
            print!("\t\t  ===============\n\n");

            match phase {
                GamePhase::Set => {
                    println!(
                        "> {}, it's your turn placing a {} stone!",
                        EMP.paint(player_name.as_str()),
                        HIGHLIGHT.paint(player_color)
                    );
                    let (stones_white, stones_black) = self.play_field.amount_of_stones;
                    println!(
                        "\n> Amount of stones on the playfield: {}: {}, {}: {}",
                        EMP.paint(&self.players.0),
                        HIGHLIGHT.paint(stones_white),
                        EMP.paint(&self.players.1),
                        HIGHLIGHT.paint(stones_black)
                    );
                    println!(
                        "> Stones left to set: {}",
                        HIGHLIGHT.paint(9 - black_rounds_done.unwrap())
                    );
                }
                GamePhase::MoveAndJump => {
                    println!(
                        "> {}, it's your turn making a move with {}!",
                        EMP.paint(player_name.as_str()),
                        HIGHLIGHT.paint(player_color)
                    );
                }
                _ => panic!(),
            }

            if !highlight.is_empty() {
                self.print_play_field_highlighted(highlight);
            } else {
                self.print_play_field();
            }
        }
        (player_color, player_name)
    }
}
