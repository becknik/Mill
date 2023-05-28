//! This module holds the representation part of the more efficient variant of the [PlayField] struct with some low-level
//! functions for accessing and modifying it's state & convert it to a canonical form, which is needed in the later parts
//! of the project.
//! It also holds some tests cases (I was to lazy to implement asserts on) & the assignment 4's test case.

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

use super::{state, PlayerColor};

mod de_encode;
mod printing;
mod win_decider;

/// Efficient representation of [PlayField] using a [u16; 3] for it's internal representation.
/// Start counting from the top middle mill field on the LSB of each u16 field for each of the 3 rectangle rings
/// The inner ring equals the index 0 in the representation array.
///
/// Using three states coded as following:
/// - 00: free
/// - 01: white
/// - 10: black
/// - 11: undefined -> assert panic!
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Hash, Default)]
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
    pub fn set_field(&mut self, ring_index: usize, index: u32, field_state: u32) {
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

        assert!(field_state < 3u32, "New field state is larger than 0x03");

        let old_ring_state = self.state[ring_index];
        // Assert target field is free, when field_state to is not
        if field_state != 0x0 {
            assert!(
                (old_ring_state & (3u16 << (index * 2))) == 0,
                "Tried to place non-free on non-free"
            );
        // Assert target field is not free, when field_state is
        } else {
            assert!(
                (old_ring_state & (3u16 << (index * 2))) != 0,
                "Tried to place free on free"
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
        //assert!(1 <= amount);
        //assert!(amount < 4);

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

    fn mirror_on_y(&mut self) {
        for ring_index in 0..3 {
            let i_1 = (3u16 << 2) & self.state[ring_index];
            let i_2 = (3u16 << 4) & self.state[ring_index];
            let i_3 = (3u16 << 6) & self.state[ring_index];

            let i_5 = (3u16 << 10) & self.state[ring_index];
            let i_6 = (3u16 << 12) & self.state[ring_index];
            let i_7 = (3u16 << 14) & self.state[ring_index];

            // Stancil out the first & fourth index
            let i_0_4 = 0b0000_0011_0000_0011u16 & self.state[ring_index];

            // Swap the game field's sides
            self.state[ring_index] =
                i_0_4 | (i_1 << 12) | (i_2 << 8) | (i_3 << 4) | (i_5 >> 4) | (i_6 >> 8) | (i_7 >> 12);
        }
    }

    /// The canonical form of EfficientPlayField ist created by selecting the length-lexicographical largest variant
    /// of the elements in the equivalent class
    // TODO &mut might be a failure... micro benchmark this!
    // pub because of benchmarking
    #[inline]
    pub fn get_canonical_form(&mut self) -> EfficientPlayField {
        let mut canonical_form = EfficientPlayField::default();

        for _i in 0..2 {
            for _j in 0..4 {
                self.mirror_on_y();
                if self > &mut canonical_form {
                    canonical_form = *self
                }

                self.mirror_on_y();
                if self > &mut canonical_form {
                    canonical_form = *self
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
        let canonical_form_1 = self.get_canonical_form();

        if canonical_form_1 == other_play_field.get_canonical_form() {
            Some(canonical_form_1)
        } else {
            None
        }
    }

    /// Calculates the possible moves of player_color, the amount of moves wich lead to a mill for player_color
    /// and the amount of stones of the other player color, which can be beaten
    ///
    /// It is possibly used as a judging function for the player agent
    #[inline]
    pub fn get_move_triple(&mut self, player_color: PlayerColor) -> (u32, u32, u32) {
        let mut moves_possible_counter = 0;
        let mut moves_to_mill_counter = 0;
        let mut stones_to_take_counter = 0;

        // Used for the extreme case when all stones of the opponent are in a mill
        let mut overall_stones_of_opposite_color_counter = 0;

        for ring_index in 0..3 {
            for field_index in (0..16).step_by(2) {
                // Current field state sifted to the LSB
                let current_field_state = (self.state[ring_index] & (3u16 << field_index)) >> field_index;

                // If the current field is empty, we wont make any adjustments to the return values
                if current_field_state == 0 {
                    continue;
                }

                // In this branch the current colors possible moves & => movements into a mill should be figured out
                if current_field_state == player_color.into() {
                    let ring_neighbors_indices = [(field_index + 14) % 16, (field_index + 18) % 16];

                    for neighbor_index in ring_neighbors_indices {
                        // Neighbor field state is empty - neighbor_index already are representational index (0 <= i < 16)
                        if (self.state[ring_index] & (3u16 << neighbor_index)) == 0 {
                            moves_possible_counter += 1;

                            moves_to_mill_counter += self.simulate_move_get_mill_count(
                                ring_index,
                                field_index,
                                MoveDirection::OnRing {
                                    target_field_index: neighbor_index,
                                },
                                current_field_state,
                            );
                        }
                    }

                    // Check for possible over-ring moves
                    if (field_index % 4) == 0 {
                        let next_rings_field_state = self.state[(ring_index + 1) % 3] & (3u16 << field_index);
                        let previous_rings_field_state = self.state[(ring_index + 2) % 3] & (3u16 << field_index);

                        match ring_index {
                            // Inner Ring
                            0 if next_rings_field_state == 0 => {
                                moves_possible_counter += 1;

                                moves_to_mill_counter += self.simulate_move_get_mill_count(
                                    0,
                                    field_index,
                                    MoveDirection::AcrossRings { target_ring_index: 1 },
                                    current_field_state,
                                )
                            }
                            // Mid Ring
                            1 => {
                                if previous_rings_field_state == 0 {
                                    moves_possible_counter += 1;

                                    moves_to_mill_counter += self.simulate_move_get_mill_count(
                                        1,
                                        field_index,
                                        MoveDirection::AcrossRings { target_ring_index: 0 },
                                        current_field_state,
                                    )
                                }

                                if next_rings_field_state == 0 {
                                    moves_possible_counter += 1;

                                    moves_to_mill_counter += self.simulate_move_get_mill_count(
                                        1,
                                        field_index,
                                        MoveDirection::AcrossRings { target_ring_index: 2 },
                                        current_field_state,
                                    )
                                }
                            }
                            // Outer Ring
                            2 if previous_rings_field_state == 0 => {
                                moves_possible_counter += 1;

                                moves_to_mill_counter += self.simulate_move_get_mill_count(
                                    2,
                                    field_index,
                                    MoveDirection::AcrossRings { target_ring_index: 1 },
                                    current_field_state,
                                )
                            }
                            _ => {}
                        }
                    }
                }
                // The opposite colors amount of stones which can be taken should be counted, which is if the stone
                // Isn't inside a mill!
                else {
                    overall_stones_of_opposite_color_counter += 1;

                    if self.get_mill_count(
                        ring_index,
                        field_index,
                        DirectionToCheck::OnAndAcrossRings {
                            player_color: current_field_state,
                        },
                    ) == 0
                    {
                        stones_to_take_counter += 1;
                    }
                }
            }
        }

        if stones_to_take_counter == 0 {
            // All stones of the opposite color are in a mill:
            stones_to_take_counter = overall_stones_of_opposite_color_counter;
        }

        (moves_possible_counter, moves_to_mill_counter, stones_to_take_counter)
    }

    /// Simulates a move of the stones of the start fields and ring index to either a it's neighboring target index or
    /// the start index on another ring, which is determined by the [MoveDirection] enum.
    ///
    /// Preconditions:
    /// - Indices should already be in "representation form" (= 0 <= x < 16).step_by(2)
    /// - The target field/ the start index on the other ring must be empty
    // TODO test if out-of-place performs better here
    fn simulate_move_get_mill_count(
        &mut self,
        start_ring_index: usize,
        start_fields_index: u32,
        direction: MoveDirection,
        color: u16,
    ) -> u32 {
        // To rollback the in-situ changes on self
        let start_ring_backup = self.state[start_ring_index];

        // Clear out the current index, must be done when simulating the moving in general
        self.state[start_ring_index] &= !(3u16 << start_fields_index);

        let mills_possible = if let MoveDirection::AcrossRings { target_ring_index } = direction {
            // To rollback the second in-situ changes on self
            let target_ring_backup = self.state[target_ring_index];

            // Setting the state of the other index, which must be empty
            self.state[target_ring_index] |= color << start_fields_index;

            // TODO makes this sense to you, future me? :|
            let mills_possible = self.get_mill_count(target_ring_index, start_fields_index, DirectionToCheck::OnRing);
            //let mills_possible = self.get_mill_count(target_ring_index, start_fields_index, DirectionToCheck::OnAndAcrossRings { player_color: color });

            // Resetting the in-place simulation on the other ring
            self.state[target_ring_index] = target_ring_backup;

            mills_possible
        } else if let MoveDirection::OnRing { target_field_index } = direction {
            // Set the empty neighbors value to the old one of the current index:
            self.state[start_ring_index] |= color << target_field_index;

            // Check for mills after the move now has taken place
            self.get_mill_count(
                start_ring_index,
                target_field_index,
                DirectionToCheck::OnAndAcrossRings { player_color: color },
            )
        } else {
            0
        };

        // Resetting the in-place simulation
        self.state[start_ring_index] = start_ring_backup;

        return mills_possible;
    }

    /// Checks for mills on the specified field & returns it.
    /// The check for mills across rings are toggled when the right argument is set. The tuple enum is there to avoid
    /// the re-calculation of the field state of the current index which should be determined on call-time
    ///
    /// Preconditions:
    /// - The field state of the current index must be not null
    /// - The fields index must be (0..16).step_by(2) and the ring index 0..3
    fn get_mill_count(&self, ring_index: usize, field_index: u32, direction: DirectionToCheck) -> u32 {
        //assert!(field_index < 16);
        //assert!(ring_index < 3);

        let mut mill_counter = 0;

        /* Rotations of the real play field anti-clockwise per index for alignment on the index 0:
        0,1 => 7
        1 => 1
        2,3 => 1
        3 => 3
        4,5 => 3
        5 => 5
        6,7 => 5
        7 => 7
        */
        let indices_to_rotate = (field_index - (field_index % 4) + 14) % 16;
        // Field state triple containing field_index:
        let state_triple = self.state[ring_index].rotate_right(indices_to_rotate) & 0b0000_0000_0011_1111u16;

        /* 010101 | 101010 */
        if state_triple == 21u16 || state_triple == 42u16 {
            mill_counter += 1;
        }

        // If index is located in an edge, two triples must be checked for mill occurrence
        if field_index == 2 || field_index == 6 || field_index == 10 || field_index == 14 {
            let state_triple = self.state[ring_index].rotate_right(field_index) & 0b000_00000_0011_1111u16;
            /* 010101 | 101010 */
            if state_triple == 21u16 || state_triple == 42u16 {
                mill_counter += 1;
            }
        }

        // Argument field index in the middle of a triple and therefore can form a mill connected to the other rings
        if let DirectionToCheck::OnAndAcrossRings { player_color } = direction {
            //assert!(color < 3);

            if field_index % 4 == 0 {
                //assert!(((self.state[ring_index] >> field_index) & 3u16) != 0);

                let next_indexs_field_state = (self.state[(ring_index + 1) % 3] & (3u16 << field_index)) >> field_index;
                let next_next_indexs_field_state =
                    (self.state[(ring_index + 2) % 3] & (3u16 << field_index)) >> field_index;

                // Mill in between rings:
                if next_indexs_field_state == player_color && next_indexs_field_state == next_next_indexs_field_state {
                    mill_counter += 1;
                }
            }
        }

        mill_counter
    }
}

/// Used by the [simulate_move_then_get_mills] method of [EfficientPlayField]
enum MoveDirection {
    OnRing { target_field_index: u32 },
    AcrossRings { target_ring_index: usize },
}

/// Used by the [get_mill_count] method of [EfficientPlayField]
enum DirectionToCheck {
    OnRing,
    OnAndAcrossRings { player_color: u16 },
}

/// Used by the [process_input_felder] method
pub enum ToWhatToProcess {
    CanonicalForm,
    MoveTripel,
}

pub fn process_input_felder(outputs_contents: ToWhatToProcess) {
    let input_felder_txt =
        File::open("input_felder.txt").expect("The 'input_felder.txt' file was not found in the projects root...");
    let reader = BufReader::new(input_felder_txt);

    let output_text = File::create("output.txt").expect("Could not create ro 'output.txt' to write results into");
    let mut writer = BufWriter::new(output_text);

    if let ToWhatToProcess::CanonicalForm = outputs_contents {
        let mut h_map: HashMap<EfficientPlayField, usize> = HashMap::new();

        for (line_index, line_content) in reader.lines().enumerate() {
            // Idk why but the reference output.txt starts counting on 1...
            let line_index = line_index + 1;

            let mut playfield = EfficientPlayField::from_coded(&line_content.unwrap());
            let canonical_form = playfield.get_canonical_form();

            if let Some(previous_canonical_match) = h_map.get(&canonical_form) {
                writeln!(writer, "{}", previous_canonical_match).unwrap();
            } else {
                h_map.insert(canonical_form, line_index);
                writeln!(writer, "{}", line_index).unwrap();
            }
        }
    } else {
        for (line_index, line_content) in reader.lines().enumerate() {
            let line_content = line_content.unwrap();
            let mut playfield = EfficientPlayField::from_coded(&line_content);

            let (x, y, z) = playfield.get_move_triple(PlayerColor::White);

            assert!({
                println!("Input {line_index}: {line_content}\n{playfield}");
                println!("Moves: {x}\nMoves->Mill: {y}\nTo Take: {z}");
                true
            });

            writeln!(writer, "{x} {y} {z}").unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EfficientPlayField;

    #[test]
    fn assignment4() {
        super::process_input_felder(super::ToWhatToProcess::CanonicalForm);
    }

    #[test]
    fn assignment5() {
        super::process_input_felder(super::ToWhatToProcess::MoveTripel);
    }

    #[test]
    fn assignment5_dbg() {
        let mut test_epf = EfficientPlayField::from_coded("BEEEWEWBEEWWEEWEWEEWWWBB");

        test_epf.get_move_triple(crate::game::PlayerColor::White);
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

            /* assert!(epf.state[2] == 0x0004);
            assert!(epf.state[1] == 0x0010);
            assert!(epf.state[1] == 0x0010); */

            epf.rotate_self_right(2);

            /* assert!(epf.state[2] == 0x0010);
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
