//! Contains everything related to the "low abstraction" of the [PlayField] printing/ painting.
use core::fmt;
use std::fmt::Display;
use std::iter::{Enumerate, Rev};
use std::slice::Iter;

use either::Either;
use smallvec::SmallVec;

use super::{FieldState, PlayField, PlayFieldError};
use crate::game::Field;
use crate::game::painting::EMP;

impl Display for FieldState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldState::Free => f.write_str("·"),
            FieldState::White => crate::game::PlayerColor::White.fmt(f),
            FieldState::Black => crate::game::PlayerColor::Black.fmt(f),
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
                    "Error caused by: {}{} - {message}",
                    erroneous_field.0, erroneous_field.1
                ))
            }
            PlayFieldError::FieldSetError {
                player: player_color,
                field,
                field_state,
                message,
            } => {
                f.write_fmt(format_args!(
                    "Error caused by setting {player_color} to field {}{} which is {} - {message}",
                    field.0, field.1, field_state
                ))
            }
            PlayFieldError::InvalidMovementError {
                start_field,
                target_field,
                player_color,
                message,
            } => {
                f.write_fmt(format_args!(
                    "Error caused by moving {player_color} from field {}{} to {}{} - {message}",
                    start_field.0, start_field.1, target_field.0, target_field.1
                ))
            }
            PlayFieldError::InvalidProgramStateError { message } => f.write_str(message),
            PlayFieldError::FailedToTake { field, message } => f.write_fmt(format_args!("Error taking field {}{} - {message}", field.0, field.1)),
        }
    }
}

type EitherIter<'a> = Either<Rev<Iter<'a, FieldState>>, Enumerate<Rev<Iter<'a, FieldState>>>>;

impl PlayField {
    /// Printing from top left to bottom right, representation in memory is left bottom to to right
    pub fn print_highlighted(&self, fields_to_highlight: Option<&[Field]>) {
        let mut iter = if fields_to_highlight.is_none() {
            EitherIter::Left(self.state.iter().rev())
        } else {
            Either::Right(self.state.iter().rev().enumerate())
        };

        let mut row_counter = 7;

        // Chose 3 because this is the maximum to highlight positions: two crossing mills is really rare...
        let mut indices_to_highlight = SmallVec::<[usize; 3]>::new();
        // Transform the array of fields into a list of representation array indices
        if fields_to_highlight.is_some() {
            for field in fields_to_highlight.unwrap() {
                let result = self.map_to_state_index(*field).unwrap();
                indices_to_highlight.push(result);
            }
        }

        let a = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let b = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let c = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        print!("\t{}|  {}------------{}------------{}\n", row_counter, c, b, a);
        row_counter -= 1;
        print!("\t |  |            |            |\n");

        let a = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let b = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let c = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        print!("\t{}|  |   {}--------{}--------{}   |\n", row_counter, c, b, a);
        row_counter -= 1;
        print!("\t |  |   |        |        |   |\n");

        let a = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let b = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let c = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        print!("\t{}|  |   |   {}----{}----{}   |   |\n", row_counter, c, b, a);
        row_counter -= 1;
        print!("\t |  |   |   |         |   |   |\n");

        let a = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let b = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let c = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let d = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let e = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let f = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        print!("\t{}|  {}---{}---{}         {}---{}---{}\n", row_counter, f, e, d, c, b, a);
        row_counter -= 1;
        print!("\t |  |   |   |         |   |   |\n");

        let a = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let b = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let c = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        print!("\t{}|  |   |   {}----{}----{}   |   |\n", row_counter, c, b, a);
        row_counter -= 1;
        print!("\t |  |   |        |        |   |\n");

        let a = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let b = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let c = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        print!("\t{}|  |   {}--------{}--------{}   |\n", row_counter, c, b, a);
        row_counter -= 1;
        print!("\t |  |            |            |\n");

        let a = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let b = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        let c = self.unwrap_and_highligth(&mut iter, &indices_to_highlight);
        print!("\t{}|  {}------------{}------------{}\n", row_counter, c, b, a);
        print!("\t   ____________________________\n");
        print!("\t    A   B   C    D    E   F   G\n");
    }

    // TODO This method is probably really inefficient...
    fn unwrap_and_highligth(&self, either_iter: &mut EitherIter, to_highlight: &[usize]) -> String {

        // Get the iter out of the Either, or just return the next of the iterator, if no highlighting shall occur
        let iter = if either_iter.is_right() {
            either_iter.as_mut().unwrap_right()
        } else {
            // Oh dear...
            return either_iter.as_mut().unwrap_left().next().unwrap().to_string();
        };
        let next_element = iter.next().unwrap();

        // The fields getting printed from top to bottom, therefore the indices must start with the last elements of state array
        let highlight_next_element = to_highlight.contains(
            &next_element.0
                .abs_diff(super::representation::constants::FIELD_COUNT - 1),
        );

        if highlight_next_element {
            EMP.paint(&next_element.1).to_string()
        } else {
            next_element.1.to_string()
        }
    }
}
