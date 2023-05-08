//! Contains everything related to the "low abstraction" of the [PlayField] printing/ painting.
use core::fmt;
use std::fmt::Display;
use std::iter::{Enumerate, Rev};
use std::slice::Iter;

use smallvec::SmallVec;

use crate::game::state::representation::constants::FIELD_COUNT;
use crate::game::PlayerColor;

use super::super::logic;

use super::representation::types::Field;
use super::{FieldState, PlayField, PlayFieldError};

impl Display for FieldState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldState::Free => f.write_str("⚬"),
            FieldState::White => PlayerColor::White.fmt(f),
            FieldState::Black => PlayerColor::Black.fmt(f),
        }
    }
}

impl Display for PlayFieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayFieldError::FieldTranslationMappingError {
                erroneous_field,
                message,
            } => {
                f.write_fmt(format_args!(
                    "Error caused by: {}{} - ",
                    erroneous_field.0, erroneous_field.1
                ))?;
                f.write_str(message)
            }
            PlayFieldError::FieldSetError {
                player,
                field,
                field_state,
                message,
            } => {
                f.write_fmt(format_args!(
                    "Error caused by setting {} to field {}{} which is {} - ",
                    player, field.0, field.1, field_state
                ))?;
                f.write_str(message)
            }
            PlayFieldError::InvalidMovementError {
                start_field,
                target_field,
                player_color,
                message,
            } => {
                f.write_fmt(format_args!(
                    "Error caused by moving {} from field {}{} to {}{} - ",
                    player_color, start_field.0, start_field.1, target_field.0, target_field.1
                ))?;
                f.write_str(message)
            }
            PlayFieldError::InvalidProgramStateError { message } => f.write_str(message),
        }
    }
}

// TODO To much code duplication...
impl PlayField {
    pub fn print_and_highlight(&self, fields_to_highlight: &[Field]) {
        let mut iter = self.state.iter().rev().enumerate();
        let mut row = 7;

        /*         let indices_to_highlight = fields_to_highlight
        .map(|field| self.map_to_state_index(field))
        .map(Result::unwrap)
        .as_slice(); */

        // Chose 5 because this is the maximum to highlight positions: two crossing mills
        let mut indices_to_highlight = SmallVec::<[usize; 5]>::new();
        for field in fields_to_highlight {
            let result = self.map_to_state_index(*field).unwrap();
            indices_to_highlight.push(result);
        }

        let a = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let b = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let c = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        print!("\t{}|  {}············{}············{}\n", row, c, b, a);
        row -= 1;
        print!("\t |  ·            ·            ·\n");

        let a = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let b = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let c = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        print!("\t{}|  ·   {}········{}········{}   ·\n", row, c, b, a);
        row -= 1;
        print!("\t |  ·   ·        ·        ·   ·\n");

        let a = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let b = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let c = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        print!("\t{}|  ·   ·   {}····{}····{}   ·   ·\n", row, c, b, a);
        row -= 1;
        print!("\t |  ·   ·   ·         ·   ·   ·\n");

        let a = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let b = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let c = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let d = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let e = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let f = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        print!("\t{}|  {}···{}···{}         {}···{}···{}\n", row, f, e, d, c, b, a);
        row -= 1;
        print!("\t |  ·   ·   ·         ·   ·   ·\n");

        let a = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let b = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let c = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        print!("\t{}|  ·   ·   {}····{}····{}   ·   ·\n", row, c, b, a);
        row -= 1;
        print!("\t |  ·   ·        ·        ·   ·\n");

        let a = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let b = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let c = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        print!("\t{}|  ·   {}········{}········{}   ·\n", row, c, b, a);
        row -= 1;
        print!("\t |  ·            ·            ·\n");

        let a = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let b = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let c = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        print!("\t{}|  {}············{}············{}\n", row, c, b, a);
        print!("\t   ____________________________\n");
        print!("\t    A   B   C    D    E   F   G\n");

        assert!(
            matches!(iter.next(), Option::None),
            "The iterator returns an element after the end of the state array print out"
        );
    }

    pub fn print_plain(&self) {
        // Printing from top left to bottom right, representation in memory is left bottom to to right
        let mut iter = self.state.iter().rev();
        let mut row = 7;

        let (a, b, c) = (iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap());
        print!("\t{}|  {}············{}············{}\n", row, c, b, a);
        row -= 1;
        print!("\t |  ·            ·            ·\n");

        let (a, b, c) = (iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap());
        print!("\t{}|  ·   {}········{}········{}   ·\n", row, c, b, a);
        row -= 1;
        print!("\t |  ·   ·        ·        ·   ·\n");

        let (a, b, c) = (iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap());
        print!("\t{}|  ·   ·   {}····{}····{}   ·   ·\n", row, c, b, a);
        row -= 1;
        print!("\t |  ·   ·   ·         ·   ·   ·\n");

        let (a, b, c) = (iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap());
        let (d, e, f) = (iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap());
        print!("\t{}|  {}···{}···{}         {}···{}···{}\n", row, f, e, d, c, b, a);
        row -= 1;
        print!("\t |  ·   ·   ·         ·   ·   ·\n");

        let (a, b, c) = (iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap());
        print!("\t{}|  ·   ·   {}····{}····{}   ·   ·\n", row, c, b, a);
        row -= 1;
        print!("\t |  ·   ·        ·        ·   ·\n");

        let (a, b, c) = (iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap());
        print!("\t{}|  ·   {}········{}········{}   ·\n", row, c, b, a);
        row -= 1;
        print!("\t |  ·            ·            ·\n");

        let (a, b, c) = (iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap());
        print!("\t{}|  {}············{}············{}\n", row, c, b, a);
        print!("\t   ____________________________\n");
        print!("\t    A   B   C    D    E   F   G\n");

        assert!(
            matches!(iter.next(), Option::None),
            "The iterator returns an element after the end of the state array print out"
        );
    }

    /// TODO This method is probably really inefficient...
    fn unwrap_and_highligth(&self, iter: &mut Enumerate<Rev<Iter<FieldState>>>, to_highlight: &[usize]) -> String {
        let next_element = iter.next().unwrap();
        // The abs_diff is necessary due to the field getting printed from top to bottom, therefore the indices must start with the upper elements
        let highlight_element = to_highlight.contains(&next_element.0.abs_diff(FIELD_COUNT - 1));

        if highlight_element {
            logic::constants::EMP.paint(&next_element.1).to_string()
        } else {
            next_element.1.to_string()
        }
    }
}
