//! This module is taught to hold everything related to the internal representation of the [PlayField] state, including methods forming abstraction from it.
use self::{
    constants::FIELD_LUT,
    types::{Field, FieldState},
};
use super::{GameFieldStateError, PlayField};

pub mod constants {

    use super::super::Field;

    pub const FIELD_COUNT: usize = 24;

    #[rustfmt::skip]
    pub const FIELD_LUT: [Field; FIELD_COUNT] = [
        ('A',1), ('D',1), ('G',1),
        ('B',2), ('D',2), ('F',2),
        ('C',3), ('D',3), ('E',3),
        ('A',4), ('B',4), ('C',4), ('E',4), ('F',4), ('G',4),
        ('C',5), ('D',5), ('E',5),
        ('B',6), ('D',6), ('F',6),
        ('A',7), ('D',7), ('G',7),
    ];
}

pub mod types {
    pub type Field = (char, u16);

    #[derive(Copy, Clone)]
    pub enum FieldState {
        Free = 0b11,
        White = 0b10,
        Black = 0b01,
    }
}

impl PlayField {
    /// Maps the player visible fields notation to the internal errors state.
    ///
    /// Handles following extreme cases:
    /// - The input character is not upper case - this is internally converted to uppercase by default
    /// -> The input character can't be converted to upper case
    /// - No array position of the LUT fits the input
    ///
    /// TODO: Calculate the uppercase by using arithmetic on chars rather than the idiomatic way
    pub fn map_to_state_index(&self, pos: Field) -> Result<usize, GameFieldStateError> {
        let as_uppercase = if let Some(c) = pos.0.to_uppercase().next() {
            c
        } else {
            return Err(GameFieldStateError::FieldTranslationMappingError {
                erroneous_field: pos,
                message: "Specified position character is out of field sizes bounds.",
            });
        };
        let pos = (as_uppercase, pos.1);

        let pos_index = FIELD_LUT
            .iter()
            .enumerate()
            .find_map(|(i, &lut_pos)| if lut_pos == pos { Some(i) } else { None });

        match pos_index {
            Some(i) => Ok(i),
            None => Err(GameFieldStateError::FieldTranslationMappingError {
                erroneous_field: pos,
                message: "Specified field is no valid game field.",
            }),
        }
    }

    /// Internally calls the [map_to_state_index] method
    pub fn get_status_of(&self, pos: Field) -> Result<FieldState, GameFieldStateError> {
        let index = self.map_to_state_index(pos)?;
        Ok(self.state[index])
    }

    // Simply calls the [map_to_state_index] function & performs a swap on the internal [state] array
    pub fn swap(&mut self, start_pos: Field, target_pos: Field) -> Result<(), GameFieldStateError> {
        let start_index = self.map_to_state_index(start_pos)?;
        let target_index = self.map_to_state_index(target_pos)?;

        self.state[target_index] = self.state[start_index];
        self.state[start_index] = FieldState::Free;

        Ok(())
    }
}
