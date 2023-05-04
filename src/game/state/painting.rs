//! Contains everything related to the "low abstraction" of the [PlayField] printing/ painting.
use core::fmt;

use super::{FieldState, PlayField};

impl fmt::Display for FieldState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            FieldState::Free => "◯",
            FieldState::White => "⚪",
            FieldState::Black => "⚫",
        })
    }
}

impl PlayField {
    pub fn print(&self) {
        let mut iter = self.state.iter();
        println!(
            "{}············{}············{}",
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap()
        );
        println!("·            ·            ·");
        println!(
            "·   {}········{}········{}   ·",
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap()
        );
        println!("·   ·        ·        ·   ·");
        println!(
            "·   ·   {}····{}····{}   ·   ·",
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap()
        );
        println!("·   ·   ·         ·   ·   ·");
        println!(
            "{}···{}···{}         {}···{}···{}",
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap()
        );
        println!("·   ·   ·         ·   ·   ·");
        println!(
            "·   ·   {}····{}····{}   ·   ·",
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap()
        );
        println!("·   ·        ·        ·   ·");
        println!(
            "·   {}········{}········{}   ·",
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap()
        );
        println!("·            ·            ·");
        println!(
            "{}············{}············{}",
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap()
        );
    }
}
