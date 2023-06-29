//! This module is taught to hold everything related to the internal representation of the [PlayField] state, including methods forming abstraction from it.
use smallvec::SmallVec;

use self::{constants::FIELD_LUT, types::FieldState};
use super::{PlayField, PlayFieldError};

use crate::game::Field;

pub mod constants {
    use crate::game::Field;

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

    #[derive(Copy, Clone, PartialEq, Debug)]
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
    /// (- The input character is not upper case - this is internally converted to uppercase by default)
    /// (-> The input character can't be converted to upper case)
    /// - This is handled by the input aquireing method
    /// - No array position of the LUT fits the input
    pub fn map_to_state_index(&self, pos: Field) -> Result<usize, PlayFieldError> {
        /*
        let as_uppercase = if let Some(c) = pos.0.to_uppercase().next() { c } else {
            return Err(PlayFieldError::FieldTranslationMappingError {
                erroneous_field: pos,
                message: "Specified position character is not a valid column on the play field.",
            });
        };
        let pos = (as_uppercase, pos.1);
        */

        assert!(pos.0.is_uppercase());
        assert!(('A'..='G').contains(&pos.0));

        let pos_index = FIELD_LUT.iter().position(|lut_element| lut_element.0 == pos.0 && lut_element.1 == pos.1);
        /*  .enumerate() .find_map(|(i, &lut_pos)| if lut_pos == pos { Some(i) } else { None }); */

        match pos_index {
            Some(i) => Ok(i),
            None => Err(PlayFieldError::FieldTranslationMappingError {
                erroneous_field: pos,
                message: "Specified field is no valid game field.",
            }),
        }
    }

    /// Internally calls the [map_to_state_index] method
    pub fn get_status_of(&self, pos: Field) -> Result<FieldState, PlayFieldError> {
        let index = self.map_to_state_index(pos)?;
        Ok(self.state[index])
    }

    // Simply calls the [map_to_state_index] function & performs a swap on the internal [state] array
    pub fn swap(&mut self, start_pos: Field, target_pos: Field) -> Result<(), PlayFieldError> {
        let start_index = self.map_to_state_index(start_pos)?;
        let target_index = self.map_to_state_index(target_pos)?;

        self.state.swap(start_index, target_index);
        Ok(())
    }

    /// Determines if a mill happened around the provided [last_updated_field] by first checking for a mill
    /// in the vertical relative triple of the [PlayField] state, and then for the same in the horizontal relative triple
    ///
    /// Returns a [SmallVec] due to the extreme case of two mills at once. & me being lazy/ confused with the return value modelling.
    pub fn get_mill_crossing(&self, last_updated_field: Field) -> SmallVec<[Field; 3]> {
        // 5 because extreme case: two mills are detected
        let mut r#return = SmallVec::<[Field; 3]>::new();

        // First: Check if mill exists on the horizontals, which is easy due to the representation as array
        let index = self.map_to_state_index(last_updated_field).unwrap();
        let position_in_triple_hor = index % 3;

        let relative_triple_state = &self.state[index - position_in_triple_hor..index + 3 - position_in_triple_hor];
        // equals:
        /* let relative_tripel = match position_in_tripel {
            0 => &self.state[position_in_tripel..position_in_tripel + 3],
            1 => &self.state[position_in_tripel - 1..position_in_tripel + 2],
            2 => &self.state[position_in_tripel - 2..position_in_tripel + 1],
            _ => panic!("Programmer's dumb lol")
        }; */

        let row = last_updated_field.1 - 1;
        let column = (last_updated_field.0 as u8) - b'A';

        if FieldState::Free != relative_triple_state[0]
            && relative_triple_state[0] == relative_triple_state[1]
            && relative_triple_state[1] == relative_triple_state[2]
        {
            let hor_coords_for_column = match row {
                0 | 6 => ('A', 'D', 'G'),
                1 | 5 => ('B', 'D', 'F'),
                2 | 4 => ('C', 'D', 'E'),
                3 if column < 4 => ('A', 'B', 'C'),
                3 if 4 <= row => ('E', 'F', 'G'),
                _ => panic!(),
            };
            let field_1 = (hor_coords_for_column.0, last_updated_field.1);
            let field_2 = (hor_coords_for_column.1, last_updated_field.1);
            let field_3 = (hor_coords_for_column.2, last_updated_field.1);
            r#return.push(field_1);
            r#return.push(field_2);
            r#return.push(field_3);
        }

        let vert_coords_for_column = match column {
            0 | 6 => (1, 4, 7),
            1 | 5 => (2, 4, 6),
            2 | 4 => (3, 4, 5),
            3 if row < 4 => (1, 2, 3),
            3 if 4 <= row => (5, 6, 7),
            _ => panic!(),
        };

        let field_1 = (last_updated_field.0, vert_coords_for_column.0);
        let field_2 = (last_updated_field.0, vert_coords_for_column.1);
        let field_3 = (last_updated_field.0, vert_coords_for_column.2);
        let relative_triple_state = (
            self.get_status_of(field_1).unwrap(),
            self.get_status_of(field_2).unwrap(),
            self.get_status_of(field_3).unwrap(),
        );

        if FieldState::Free != relative_triple_state.0
            && relative_triple_state.0 == relative_triple_state.1
            && relative_triple_state.1 == relative_triple_state.2
        {
            r#return.push(field_1);
            r#return.push(field_2);
            r#return.push(field_3);
        }

        r#return

        // Second: Check if mill occurs on the verticals, a bit more tricky
        //let vertical_position = (last_update.0 as u8) - ('A' as u8);
        // The amount of coordiantes until the next member of the column is reached
        /* let moves_for_layer = match vertical_position {
            0 | 6 => 3,
            1 | 5 => 2,
            2 | 4 => 1,
            3 => 1,
            _ => panic!("Programmer's dumb lol"),
        }; */
        //let position_in_coord_tripel = (last_update.0 as u8) - ('1' as u8);
        // Subtracted form the position_in_coord_tripel to make the button number align to 0
        // Needed to divide the result with the moves_for_layer, to gain the position in the tripel form 0 to 3
        /*let inlay = match vertical_position {
            0 | 6 => 1,
            1 | 5 => 2,
            2 | 4 => 1,
            3 => 0,
            _ => panic!("Programmer's dumb lol"),
        };*/
        //let position_in_tripel = (position_in_coord_tripel - inlay) / moves_for_layer;
        // Special case: in the middle/ column D, there are two tuples
        //let position_in_tripel: u8 = if 3 < position_in_tripel {position_in_tripel - 4} else {position_in_tripel};

        //let coord_range = position_in_coord_tripel - (position_in_tripel * moves_for_layer) + 1.. position_in_coord_tripel + ((position_in_tripel - 2) * moves_for_layer) + 1;
    }

    /// Takes a stone form the specified field
    pub fn take(&mut self, field: Field) {
        let index = self.map_to_state_index(field).unwrap();
        let stone_to_take = self.state[index];
        self.state[index] = FieldState::Free;

        match stone_to_take {
            FieldState::White => self.amount_of_stones.0 -= 1,
            FieldState::Black => self.amount_of_stones.1 -= 1,
            FieldState::Free => panic!(),
        }
    }
}
