pub mod printing;
pub mod representation;

use self::representation::types::*;
use crate::game::Field;

#[derive(Debug)]
pub enum PlayFieldError {
    FieldTranslationMappingError {
        erroneous_field: Field,
        message: &'static str,
    },
    FieldSetError {
        player: PlayerColor,
        field: Field,
        field_state: FieldState,
        message: &'static str,
    },
    InvalidMovementError {
        start_field: Field,
        target_field: Field,
        player_color: PlayerColor,
        message: &'static str,
    },
    InvalidProgramStateError {
        message: &'static str,
    },
    FailedToTake {
        field: Field,
        message: &'static str,
    },
}

use self::representation::constants::*;

use super::PlayerColor;

pub struct PlayField {
    state: [FieldState; FIELD_COUNT],
    // first one: white, second one: black
    pub amount_of_stones: (u32, u32),
}

impl Default for PlayField {
    fn default() -> Self {
        Self {
            state: [FieldState::Free; FIELD_COUNT],
            amount_of_stones: (0, 0),
        }
    }
}

impl PlayField {
    /// Sets a sone to the specified position by calling the [get_status_of] method.
    /// Then modifies the interior [state] array
    ///
    /// Handled extreme cases:
    /// - The selected field is not empty
    pub fn try_set(&mut self, pos: Field, color: PlayerColor) -> Result<(), PlayFieldError> {
        use FieldState::*;

        assert!(self.amount_of_stones.0 < 10 && self.amount_of_stones.1 < 10);

        let current_state = self.get_status_of(pos)?;
        if matches!(current_state, Free) {
            let index_to_change = self.map_to_state_index(pos)?;

            self.state[index_to_change] = color.into();
            match color {
                PlayerColor::White => self.amount_of_stones.0 += 1,
                PlayerColor::Black => self.amount_of_stones.1 += 1,
            }
            Ok(())
        } else {
            Err(PlayFieldError::FieldSetError {
                player: color,
                field: pos,
                field_state: current_state,
                message: "Stone must be placed upon free field.",
            })
        }
    }

    /// First method called when a player tries to move a stone from one field to another.
    /// It permits the move by calling self.move if the amount of stones == 3 or if the move occurs vertically & exactly one field difference.
    ///
    /// Handles the move in context of the game state:
    /// - If the player has more than 3 stones, it should not be possible to jump
    /// - The stone can't be moved to it's own field
    /// - The stone can't be moved more than one field in a direction
    /// - All other states are not allowed
    pub fn try_move(&mut self, start_pos: Field, target_pos: Field, color: PlayerColor) -> Result<(), PlayFieldError> {
        let players_stone_count = match color {
            PlayerColor::White => self.amount_of_stones.0,
            PlayerColor::Black => self.amount_of_stones.1,
        };

        if let Err(err) = self.map_to_state_index(start_pos) {
            return Err(err);
        } else if let Err(err) = self.map_to_state_index(target_pos) {
            return Err(err);
        }

        // Jumps, with more than 3 stones
        if 4 <= players_stone_count && start_pos.0 != target_pos.0 && start_pos.1 != target_pos.1 {
            Err(PlayFieldError::InvalidMovementError {
                start_field: start_pos,
                target_field: target_pos,
                player_color: color,
                message: "The movement of a stone must occur horizontally or vertically.",
            })
        // Move to same field
        } else if start_pos.0 == target_pos.0 && start_pos.1 == target_pos.1 {
            Err(PlayFieldError::InvalidMovementError {
                start_field: start_pos,
                target_field: target_pos,
                player_color: color,
                message: "The stone can't stay on the same field after moving.",
            })
        // Jumps
        } else if players_stone_count == 3 {
            self.r#move(start_pos, target_pos, color)
        // Moves in one direction
        } else if (start_pos.0 == target_pos.0) ^ (start_pos.1 == target_pos.1) {
            // if the letter stays the same, the letter is constant & specifies the layer the move should take place
            let (layer_to_move_on_as_u8, constant_coord) = if start_pos.0 == target_pos.0 {
                ((start_pos.0 as u8) - b'A', false)
            } else {
                (start_pos.1 - 1, true)
            };
            //println!("layer: {}, constant coord: {}", layer_to_move_on_as_u8, constant_coord);

            let legal_move_range = match layer_to_move_on_as_u8 {
                0 | 6 => 3,
                1 | 5 => 2,
                2 | 4 => 1,
                3 => 1, // <- took an hour or so
                _ => panic!(),
            };
            // println!("Legal move range: {}", legal_move_range);

            match constant_coord {
                false => {
                    //println!("Calculating delta: {}", (start_pos.1 as i16 - target_pos.1 as i16).abs());
                    if ((start_pos.1 as i16) - (target_pos.1 as i16)).unsigned_abs() != legal_move_range {
                        return Err(PlayFieldError::InvalidMovementError {
                            start_field: start_pos,
                            target_field: target_pos,
                            player_color: color,
                            message: "Stone can't be moved multiple fields vertical (letter is constant) ahead.",
                        });
                    }
                }
                true => {
                    //println!("Calculating delta: {}", (start_pos.0 as i16 - target_pos.0 as i16).abs());
                    if (start_pos.0 as i16 - target_pos.0 as i16).unsigned_abs() != legal_move_range {
                        return Err(PlayFieldError::InvalidMovementError {
                            start_field: start_pos,
                            target_field: target_pos,
                            player_color: color,
                            message: "Stone can't be moved multiple fields horizontal (number is constant) ahead.",
                        });
                    }
                }
            }

            // Exactly one move
            self.r#move(start_pos, target_pos, color)
        // Any other?
        } else {
            Err(PlayFieldError::InvalidProgramStateError {
                message: "State should never be reached in try_move method. Might be caused an stone count < 3. Programmer's dumb.",
            })
        }
    }

    /// As the other try_... function, this one also checks cases in the context of the player's color &
    /// if it's permitted to proceed taking the specified stone
    pub fn try_take(&mut self, field_to_take: Field, player_color: PlayerColor) -> Result<(), PlayFieldError> {
        let field_state = match self.get_status_of(field_to_take) {
            Ok(state) => state,
            Err(_) => {
                return Err(PlayFieldError::FailedToTake {
                    field: field_to_take,
                    message: "Specified field is no valid game field",
                })
            }
        };

        if field_state != player_color.into() && field_state != FieldState::Free {
            // If the field to take is in a mill
            if !self.get_mill_crossing(field_to_take).is_empty() {
                return Err(PlayFieldError::FailedToTake {
                    field: field_to_take,
                    message: "The specified stone to take is in at lease one mill.",
                });
            }
            self.take(field_to_take);
            Ok(())
        } else {
            Err(PlayFieldError::FailedToTake {
                field: field_to_take,
                message: "The specified field must be covered with an opponent stone.",
            })
        }
    }
}

impl PlayField {
    // Handles the move in context of the state of the game field, covering the following extreme cases:
    // - The start field doesn't contain a stone of the players color
    // - The target field isn't empty
    fn r#move(&mut self, start_pos: Field, target_pos: Field, color: PlayerColor) -> Result<(), PlayFieldError> {
        let start_status = self.get_status_of(start_pos)?;
        let target_status = self.get_status_of(target_pos)?;

        if start_status == color.into() {
            // Target field == free
            if matches!(target_status, FieldState::Free) {
                self.swap(start_pos, target_pos)
            } else {
                Err(PlayFieldError::InvalidMovementError {
                    start_field: start_pos,
                    target_field: target_pos,
                    player_color: color,
                    message: "Specified target field is not free.",
                })
            }
        // Start field empty of != color
        } else {
            Err(PlayFieldError::InvalidMovementError {
                start_field: start_pos,
                target_field: target_pos,
                player_color: color,
                message: match start_status {
                    FieldState::Free => "Blank fields can't be moved.",
                    FieldState::White | FieldState::Black => "Stone in the opposite color can't be moved.",
                },
            })
        }
    }
}
