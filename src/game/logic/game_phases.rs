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
    /// Returns valid coordinates of the game field in A_G, 1-7 mapping. The coordinate is requested after printing out the message argument
    /// Loops & requests input until the provided input is valid. Handles ALL error cases.
    ///
    /// Handled extreme cases:
    /// - Input fails
    /// - Input is to short or to long
    /// - first char is not \in 'A'-'G'
    /// - second char is not \in 1-7
    pub fn get_field_coord_input(&self, message: &str) -> Field {
        return loop {
            print!("{}", message);
            io::stdout().flush().unwrap();

            let mut input_buffer = String::new();

            match io::stdin().read_line(&mut input_buffer) {
                Ok(_) => {
                    let input_buffer = input_buffer.trim();

                    if input_buffer.len() < 2 {
                        print_error("Provided input is to short.");
                        continue;
                    } else if 3 <= input_buffer.len() {
                        print_error("Provided input is longer than 2 characters.")
                    }

                    // Parsing checks
                    let row_char = match input_buffer[0..1].parse::<char>() {
                        Ok(c) if ('A'..='G').contains(&c) => c.to_uppercase().next().unwrap(),
                        Ok(_) => {
                            print_error("Provided input character isn't between A - G.");
                            continue;
                        }
                        Err(_) => {
                            print_error("Input does't start with a letter representing a column.");
                            continue;
                        }
                    };
                    let column_char = match input_buffer[1..2].parse::<u8>() {
                        Ok(n) if (1..=7).contains(&n) => n,
                        Ok(_) => {
                            print_error("Second input char is not 1 <= && < 8.");
                            continue;
                        }
                        Err(_) => {
                            print_error("Second input char is not a number. Input is ill formatted.");
                            continue;
                        }
                    };

                    break (row_char, column_char);
                }
                Err(error) => print_error(&format!("Error occurred processing input: {error}",)),
            }
        }
    }

    /// Returns if mills were detected & returns them if so and prints them out
    pub fn check_for_and_get_mils(&self, last_updated_field: Field) -> Option<SmallVec<[Field; 3]>> {

        let mills = self.play_field.get_mill_crossing(last_updated_field);

        // This hurts. And I'm not sure how to do better.
        if mills.len() == 0 {
            None
        } else if mills.len() == 3 {
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
        } else {
            assert!(mills.len() == 6);

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
        }
    }

    /// Handles the mill cross-check of the last field a stone was set upon.
    /// Includes the user interaction part for selecting a valid field on the [PlayField].
    /// Handled extreme cases:
    /// - All stones on the play field are element of mills
    /// TODO This is to weak. If the player e.g. has 3 stones & all are in a mill, it must be skipped too...
    ///
    /// Returns true if a mill was detected for the [GamePhase] cases to trigger coordinative behavior.
    pub fn do_mills_interaction(
        &mut self,
        input_field: (char, u8),
        player_color: crate::game::PlayerColor,
    ) -> Option<SmallVec<[Field; 3]>> {
        if let Some(mills) = self.check_for_and_get_mils(input_field) {

            self.print_play_field_highlighted(&mills);

            // A piece of a extreme case:
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

    /// Prints (depending of the state of [GameCoordinator]) out the current round, the state of the play field and messages for some phases of [GamePhase].
    /// Also skips this print outs, if the provided [error_occurred] is true.
    /// Returns some convenient values needed in the game phases for coordination of the [PlayField].
    pub fn print_turn_header(
        &self,
        phase: GamePhase,
        black_rounds_done: Option<u32>,
        highlight: &[Field],
    ) -> (crate::game::PlayerColor, smartstring::SmartString<smartstring::Compact>) {
        let (player_name, player_color) = self.get_current_turns_attributes();
        let player_name = CompactString::from(player_name);

        // Print out the round and game field info, if no error occurred
        if !self.error_state {
            println!("\n\n\t\t  ===============");
            println!("\t\t  === {} ===", HIGHLIGHT.paint(format!("Round {}", self.round)));
            println!("\t\t  ===============\n");

            if let GamePhase::Set = phase {
                println!(
                    "> {}, it's your turn placing a {} stone!",
                    EMP.paint(player_name.as_str()),
                    HIGHLIGHT.paint(player_color)
                );
                let (stones_white, stones_black) = self.play_field.amount_of_stones;
                println!(
                    "\n> Amount of stones on the playfield: {}: {}, {}: {}",
                    EMP.paint(&self.player_names.0),
                    HIGHLIGHT.paint(stones_white),
                    EMP.paint(&self.player_names.1),
                    HIGHLIGHT.paint(stones_black)
                );
                println!(
                    "> Stones left to set: {}",
                    HIGHLIGHT.paint(9 - black_rounds_done.unwrap())
                );
            } else if let GamePhase::MoveAndJump = phase {
                println!(
                    "> {}, it's your turn making a move with {}!",
                    EMP.paint(player_name.as_str()),
                    HIGHLIGHT.paint(player_color)
                );
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
