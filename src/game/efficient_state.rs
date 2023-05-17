//! This module holds the representation part of the more efficient variant of the [PlayField] struct with some low-level
//! functions for accessing and modifying it's state & convert it to a canonical form, which is needed in the later parts
//! of the project.
//! It also holds some tests cases (I was to lazy to implement asserts on) & the assignment 4's test case.

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

mod de_encode;
mod printing;

/// Efficient representation of [PlayField] using a [u16; 3] for it's internal representation.
/// Start counting from the top middle mill field on the LSB of each u16 field for each of the 3 rectangle rings
/// The inner ring equals the index 0 in the representation array.
///
/// Using three states coded as following:
/// - 00: free
/// - 01: white
/// - 10: black
/// - 11: undefined -> assert panic!
#[derive(Clone, Eq, PartialEq, PartialOrd, Hash, Default)]
pub struct EfficientPlayField {
    state: [u16; 3],
}

pub struct EfficientPlayField4 {
    states: [u16; 12],
}

impl EfficientPlayField {
    /// Sets the field according to the binary parameters. The indices are specified binary coded
    ///
    /// Handled extreme cases:
    /// - Ensures that black or white fields are replaced by free, vice versa
    /// - The input parameters must have values that make sense, ring_index < 3, index < 8, state < 3
    fn set_field(&mut self, ring_index: usize, index: u32, field_state: u32) {
        // Ensures no 11 exists in the state array
        assert!(
            {
                let invariant_hurt = self.assert_state_invariant();
                if let Some((ring_index, rect_index)) = invariant_hurt {
                    eprintln!("11 on ring index {ring_index}, rect_index {rect_index}")
                }
                invariant_hurt.is_none()
            },
            "States invariant is hurt."
        );

        assert!(ring_index < 3usize, "Ring index is larger than 0x03");

        assert!(index < 8u32, "Index is greater than 0x07");

        assert!(field_state < 3u32, "New field state is larger than 0x04");

        let old_ring_state = self.state[ring_index];
        // Assert target field is free, when field_state to is not
        if field_state != 0x0 {
            assert!(
                (old_ring_state & (0x3 << (index * 2))) == 0x0,
                "Tried to place non-free on non-free"
            );
        // Assert target field is not free, when field_state is
        } else {
            assert!(
                (old_ring_state & (0x3 << (index * 2))) != 0x0,
                "Tried to place white on free on free"
            );
        }

        // Shifting mask upon field index & applying it with disjunction
        let new_state_mask = (field_state as u16) << (index * 2);
        self.state[ring_index] = old_ring_state | new_state_mask;
    }

    /// Validates the invariant that no 11 might occur in any position of the array.
    /// If the state array contains such state, this method returns the fields index (ring_index, rect_index)
    fn assert_state_invariant(&self) -> Option<(usize, u8)> {
        for ring_index in 0..3 {
            let ring_state = self.state[ring_index];

            for i in (0..16).step_by(2) {
                if (ring_state & (0x03 << i)) == (0x03 << i) {
                    return Some((ring_index, i));
                }
            }
        }
        None
    }
    /// Rotates the rings of the mill in the right direction
    fn rotate_self_right(&mut self, amount: u32) {
        assert!(1 <= amount);
        assert!(amount < 4);

        for ring_index in 0..3 {
            // Due to the ring representation staring on the LSB, we have to shift left 2 fields internally, which
            // equals 4 bits in total
            self.state[ring_index] = self.state[ring_index].rotate_left(2 * 2 * amount);
        }
    }

    /// Swaps the inner ring/ rect in place with the outer ring
    fn swap_rings(&mut self) {
        self.state.swap(0, 2);

        /* let buff = self.state[0];
        self.state[0] = self.state[2];
        self.state[2] = buff; */
    }

    // TODO use the be/le methods from primitives on slices for this?
    fn mirror_on_y(&mut self) {
        for ring_index in 0..3 {
            let i_1 = 0b0000000000001100 & self.state[ring_index];
            let i_2 = 0b0000000000110000 & self.state[ring_index];
            let i_3 = 0b0000000011000000 & self.state[ring_index];

            let i_5 = 0b0000110000000000 & self.state[ring_index];
            let i_6 = 0b0011000000000000 & self.state[ring_index];
            let i_7 = 0b1100000000000000 & self.state[ring_index];

            let i_0_4 = 0b0000001100000011 & self.state[ring_index];

            self.state[ring_index] = i_0_4
                | (i_1 << (6 * 2))
                | (i_2 << (4 * 2))
                | (i_3 << (2 * 2))
                | (i_5 >> (2 * 2))
                | (i_6 >> (4 * 2))
                | (i_7 >> (6 * 2));

            /*
            let start_index = 1;
            for delta in (6..=2).step_by(2) {
                let current_right_fields_val = self.state[ring_index] & (0x3 << 2 * start_index);
                self.state[ring_index] = (self.state[ring_index] ^ (0x03 << ((start_index * 2) + delta))) | (current_right_fields_val << delta);
            } */
            //let left_and_right_indices_state = self.state[i] & 0b1111110011111100;
            //self.state[i] = (self.state[i] & 0b0000001100000011) | (left_and_right_indices_state >> 8) | (left_and_right_indices_state << 8);
        }
    }

    /// The canonical form of EfficientPlayField ist created by selecting the length-lexicographical largest variant
    /// of the elements in the equivalent class
    // TODO &mut might be a failure... micro benchmark this!
    fn get_canonical_form(&mut self) -> EfficientPlayField {
        let mut canonical_form = EfficientPlayField { state: [0x0; 3] };

        for _i in 0..2 {
            for _j in 0..4 {
                self.mirror_on_y();
                if self > &mut canonical_form {
                    canonical_form = self.clone()
                }

                self.mirror_on_y();
                if self > &mut canonical_form {
                    canonical_form = self.clone()
                }

                self.rotate_self_right(1);
            }
            self.swap_rings();
        }

        canonical_form
    }

    /// Checks weather the current field is in the same equivalence class as the other play field by calling [get_canonical_form]
    /// on both play fields and the comparing the result
    fn in_same_equivalence_class_as(
        &mut self,
        other_play_field: &mut EfficientPlayField,
    ) -> Option<EfficientPlayField> {
        let canoncial_form_1 = self.get_canonical_form();

        if canoncial_form_1 == other_play_field.get_canonical_form() {
            Some(canoncial_form_1)
        } else {
            None
        }
    }
}

fn process_input_felder_txt() {
    let input_felder_txt =
        File::open("input_felder.txt").expect("The 'input_felder.txt' file was not found in the projects root...");
    let reader = BufReader::new(input_felder_txt);

    let output_text = File::create("output.txt").expect("Could not create ro 'output.txt' to write results into");
    let mut writer = BufWriter::new(output_text);

    let mut h_map: HashMap<EfficientPlayField, usize> = HashMap::new();

    for (line_index, line_content) in reader.lines().enumerate() {
        // Idk why but the reference output.txt starts counting on 1...
        let line_index = line_index + 1;

        let mut test_playfield = EfficientPlayField::from_coded(&line_content.unwrap());
        let canonical_form = test_playfield.get_canonical_form();

        if let Some(previous_canoncial_match) = h_map.get(&canonical_form) {
            writeln!(writer, "{}", previous_canoncial_match).unwrap();

            /*writer
            .write_fmt(format_args!("{}\n", previous_canoncial_match))
            .unwrap(); */
        } else {
            h_map.insert(canonical_form, line_index);
            writeln!(writer, "{}", line_index).unwrap();

            //writer.write_fmt(format_args!("{}\n", line_index)).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EfficientPlayField;

    #[test]
    fn assignment() {
        super::process_input_felder_txt();
    }

    mod normal {
        use super::*;

        #[test]
        fn set_field_normal() {
            let mut epf = EfficientPlayField::default();

            epf.set_field(2, 7, 2); // ring 2, index 7, to black
            epf.set_field(1, 7, 1); // ring 1, index 7, to white
            epf.set_field(0, 7, 1); // ring 0, index 7, to white

            epf.set_field(1, 0, 2); // ring 1, index 0, to black
            epf.set_field(1, 1, 2); // ring 1, index 1, to black
            epf.set_field(1, 2, 2); // ring 1, index 2, to black
            epf.set_field(1, 3, 2); // ring 1, index 3, to black
            epf.set_field(1, 4, 2); // ring 1, index 4, to black
            epf.set_field(1, 5, 2); // ring 1, index 5, to black
            epf.set_field(1, 6, 2); // ring 1, index 6, to black

            epf.set_field(0, 6, 2); // ring 1, index 6, to black
            epf.set_field(2, 2, 2); // ring 1, index 6, to black
            epf.set_field(2, 4, 1); // ring 1, index 6, to black

            println!("\nAfter some added stones: {}", epf);
        }

        #[test]
        fn rotate1() {
            let mut epf = EfficientPlayField::default();

            epf.set_field(2, 0, 1);
            epf.set_field(1, 1, 2);
            epf.set_field(0, 2, 1);

            println!("\nInitial state: {}", epf);
            epf.rotate_self_right(1);
            println!("First rotation: {}", epf);
            epf.rotate_self_right(1);
            println!("Second rotation: {}", epf);
            epf.rotate_self_right(1);
            println!("Third rotation: {}", epf);

            /* 			assert!(epf.state[2] == 0x0004);
            assert!(epf.state[1] == 0x0010);
            assert!(epf.state[1] == 0x0010); */

            epf.rotate_self_right(2);

            /* 			assert!(epf.state[2] == 0x0010);
            assert!(epf.state[1] == 0x0080);
            assert!(epf.state[1] == 0x0100); */

            epf.rotate_self_right(3);
        }

        #[test]
        fn mirror1() {
            let mut epf = EfficientPlayField::default();

            epf.set_field(2, 0, 1);
            epf.set_field(1, 1, 2);
            epf.set_field(0, 2, 1);
            epf.set_field(2, 3, 2);

            println!("\nNot mirrored:{}", epf);

            epf.mirror_on_y();

            println!("Mirrored: {}", epf);
        }

        #[test]
        fn canonical1() {
            let test = "BBEEEEEBEEEEWEWWBWWEEEBE";
            println!("Input: {test}");

            let mut epf = EfficientPlayField::from_coded(test);

            println!("{}", epf);

            let epf = epf.get_canonical_form();

            println!("Output: {}", epf.to_string_representation());
        }
    }

    mod extreme {
        use super::*;

        #[test]
        #[should_panic]
        fn set_field_black_to_white() {
            let mut epf = EfficientPlayField::default();

            epf.set_field(2, 7, 2); // ring 2, index 7, to black
            epf.set_field(2, 7, 2); // ring 2, index 7, to white
        }

        #[test]
        #[should_panic]
        fn set_field_white_to_black() {
            let mut epf = EfficientPlayField::default();

            epf.set_field(2, 7, 1); // ring 2, index 7, to white
            epf.set_field(2, 7, 2); // ring 2, index 7, to black
        }

        #[test]
        #[should_panic]
        fn set_field_free_to_free() {
            let mut epf = EfficientPlayField::default();

            epf.set_field(1, 3, 0); // ring 1, index 3, to white
        }

        #[test]
        #[should_panic]
        fn set_field_to_11() {
            let mut epf = EfficientPlayField::default();

            epf.set_field(1, 3, 4); // ring 1, index 3, to undefined
        }

        #[test]
        #[should_panic]
        fn set_ring_index_to_11() {
            let mut epf = EfficientPlayField::default();

            epf.set_field(3, 1, 2); // ring ?, index 1, to black
        }
    }
}
