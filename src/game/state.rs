pub mod painting;
//mod representation; TODO
pub mod representation;

use self::representation::types::*;

pub enum GameFieldStateError {
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
        start: Field,
        target: Field,
        stone_count: u32,
        message: &'static str,
    },
    InvalidProgramStateError {
        message: &'static str,
    },
}

use self::representation::constants::*;

use super::PlayerColor;

pub struct PlayField {
    state: [FieldState; FIELD_COUNT],
    // first one: white, second one: black
    amount_of_stones: (u32, u32),
}

impl PlayField {
    pub fn new() -> PlayField {
        PlayField {
            state: [FieldState::Free; FIELD_COUNT],
            amount_of_stones: (0, 0),
        }
    }

    /// Sets a sone to the specified position by calling the [get_status_of] method.
    /// Then modifies the interior [state] array
    ///
    /// Handled extreme cases:
    /// - The selected field is not empty
    pub fn try_set(&mut self, pos: Field, color: PlayerColor) -> Result<(), GameFieldStateError> {
        use FieldState::*;

        let current_state = self.get_status_of(pos)?;
        if matches!(current_state, Free) {
            let index_to_change = self.map_to_state_index(pos)?;
            self.state[index_to_change] = match color {
                PlayerColor::White => FieldState::White,
                PlayerColor::Black => FieldState::Black,
            };
            Ok(())
        } else {
            Err(GameFieldStateError::FieldSetError {
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
    pub fn try_move(
        &mut self,
        start_pos: Field,
        target_pos: Field,
        color: PlayerColor,
    ) -> Result<(), GameFieldStateError> {
        let players_stone_count = match color {
            PlayerColor::White => self.amount_of_stones.0,
            PlayerColor::Black => self.amount_of_stones.1,
        };

        // Jumps, with more than 3 stones
        return if 4 <= players_stone_count && start_pos.0 != target_pos.0 && start_pos.1 != target_pos.1 {
            Err(GameFieldStateError::InvalidMovementError {
                start: start_pos,
                target: target_pos,
                stone_count: players_stone_count,
                message: "The movement of a stone must occur horizontally or vertically.",
            })
        // Move to same field
        } else if start_pos.0 == target_pos.0 && start_pos.1 == target_pos.1 {
            Err(GameFieldStateError::InvalidMovementError {
                start: start_pos,
                target: target_pos,
                stone_count: players_stone_count,
                message: "The stone can't stay on the same field after moving.",
            })
        // Jumps
        } else if players_stone_count == 3 {
            self.r#move(start_pos, target_pos, color, players_stone_count)
        // Moves in one direction
        } else if (start_pos.0 == target_pos.0) ^ (start_pos.1 == target_pos.1) {
            // More than one move
            if (start_pos.0 as i8 - target_pos.0 as i8).abs() >= 2
                || (start_pos.1 as i16 - target_pos.1 as i16).abs() >= 2
            {
                Err(GameFieldStateError::InvalidMovementError {
                    start: start_pos,
                    target: target_pos,
                    stone_count: players_stone_count,
                    message: "Stone can't be moved two fields ahead.",
                })
            // Exactly one move
            } else {
                self.r#move(start_pos, target_pos, color, players_stone_count)
            }
        // Any other?
        } else {
            Err(GameFieldStateError::InvalidProgramStateError {
                message: "State should never be reached in try_move method. Might be caused an stone count < 3",
            })
        };
    }
}

impl PlayField {
    // Handles the move in context of the state of the game field, covering the following extreme cases:
    // - The start field doesn't contain a stone of the players color
    // - The target field isn't empty
    fn r#move(
        &mut self,
        start_pos: Field,
        target_pos: Field,
        color: PlayerColor,
        stone_count: u32,
    ) -> Result<(), GameFieldStateError> {
        let start_status = self.get_status_of(start_pos)?;
        let target_status = self.get_status_of(target_pos)?;

        // Start field == color check
        return if (matches!(color, PlayerColor::White) && matches!(start_status, FieldState::White))
            || matches!(color, PlayerColor::Black) && matches!(start_status, FieldState::Black)
        {
            // Target field == free
            if matches!(target_status, FieldState::Free) {
                self.swap(start_pos, target_pos)
            } else {
                Err(GameFieldStateError::InvalidMovementError {
                    start: start_pos,
                    target: target_pos,
                    stone_count: stone_count,
                    message: "Target field is not free.",
                })
            }
        // Start field empty of != color
        } else {
            Err(GameFieldStateError::InvalidMovementError {
                start: start_pos,
                target: target_pos,
                stone_count: stone_count,
                message: match start_status {
                    FieldState::Free => "Player tried to move a blank field.",
                    FieldState::White | FieldState::Black => "Player tried to move stone in the opposite color.",
                },
            })
        };
    }
}
