//! This module holds the representation part of the more efficient variant of the [PlayField] struct with some low-level
//! functions for accessing and modifying it's state & convert it to a canonical form, which is needed in the later parts
//! of the project.
//! It also holds some tests cases (I was to lazy to implement asserts on) & the assignment 4's test case.

use std::{
    fs::File,
    hash::{Hash, Hasher},
    io::{BufRead, BufReader, BufWriter, Write},
};

use super::PlayerColor;

use fnv::FnvHashMap;

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
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Default)]
pub struct EfficientPlayField {
    state: [u16; 3],
}

#[derive(Clone, Copy)]
pub struct FieldPos {
    ring_index: usize,
    field_index: u16,
}

impl Hash for EfficientPlayField {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.state.hash(state)
    }
}

pub struct EfficientPlayField4 {
    states: [u16; 12],
}

enum RelativePosition {
    Previous,
    Next,
}
use smallvec::SmallVec;
use RelativePosition::*;

impl EfficientPlayField {
    /// Sets the field according to the binary parameters. The indices are specified binary coded
    ///
    /// Handled extreme cases:
    /// - Ensures that black or white fields are replaced by free, vice versa
    /// - The input parameters must have values that make sense, ring_index < 3, index < 8, state < 3
    pub fn set_field_state(&mut self, ring_index: usize, index: u32, field_state: u32) {
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

    /// Bitmasks out the specified field and returns it either aligned to the LSB or unaligned as masked out.
    ///
    /// The field_index must be < 8!
    fn get_field_state_at(&self, ring_index: usize, field_index: u16, align_to_lsb: bool) -> u16 {
        assert!(
            field_index < 8,
            "`get_field_state_at` makes use of an abstract index representation"
        );

        let field_state = self.state[ring_index] & (3u16 << (field_index * 2));
        return if align_to_lsb {
            field_state >> (field_index * 2)
        } else {
            field_state
        };
    }

    fn get_ring_index(ring_index: usize, relative: RelativePosition) -> usize {
        return (ring_index
            + match relative {
                RelativePosition::Previous => 2,
                RelativePosition::Next => 1,
            })
            % 3;
    }

    /// Returns the (index in abstract form, state) of the on-ring neighbor of the specified field.
    /// No alignment to the LSB is done.
    fn get_neighbor_field_states(&self, ring_index: usize, field_index: u16) -> [(u16, u16); 2] {
        let indices = [(field_index + 7) % 8, (field_index + 1) % 8];

        return [
            (indices[0], self.get_field_state_at(ring_index, indices[0], false)),
            (indices[1], self.get_field_state_at(ring_index, indices[1], false)),
        ];
    }

    /// Validates the invariant that no 11 might occur in any position of the array.
    /// If the state array contains such state, this method returns the fields index (ring_index, rect_index)
    fn assert_state_invariant(&self) -> Option<(usize, u16)> {
        for ring_index in 0..3 {
            let _ring_state = self.state[ring_index];

            for field_index in 0..8 {
                if self.get_field_state_at(ring_index, field_index, true) == 3u16 {
                    return Some((ring_index, field_index));
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
            // Due to rep staring on the LSB, we have to shift left
            // 2 fields to shift, which equals 4 bits in total
            self.state[ring_index] = self.state[ring_index].rotate_left(2 * 2 * amount);
        }
    }

    /// Swaps the inner ring/ rect in place with the outer ring
    fn swap_rings(&mut self) {
        self.state.swap(0, 2);
    }

    fn mirror_on_y(&mut self) {
        for ring_index in 0..3 {
            let i_1 = self.get_field_state_at(ring_index, 1, false);
            let i_2 = self.get_field_state_at(ring_index, 2, false);
            let i_3 = self.get_field_state_at(ring_index, 3, false);

            let i_5 = self.get_field_state_at(ring_index, 5, false);
            let i_6 = self.get_field_state_at(ring_index, 6, false);
            let i_7 = self.get_field_state_at(ring_index, 7, false);

            // Mask the first & fourth index
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
    //#[inline]
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

    /// Calculates the possible moves of color, the amount of moves wich lead to a mill for color
    /// and the amount of stones of the other players color, which can be beaten
    ///
    /// It is possibly used as a judging function for the player agent
    //#[inline]
    pub fn get_move_triple(&mut self, color: PlayerColor) -> (u32, u32, u32) {
        let mut moves_possible_counter: u32 = 0;
        let mut moves_into_mill_counter: u32 = 0;
        let mut stones_to_take_counter: u32 = 0;

        // Used for the extreme case when all stones of the opponent are in a mill
        let mut overall_stones_of_opposite_color_counter = 0;

        let (color_positions, not_color_positions) = self.get_positions(color);
        if color_positions.len() == 3 {
            moves_possible_counter = 3 * not_color_positions.len() as u32;
            stones_to_take_counter = not_color_positions.len() as u32;

            for position in not_color_positions {
                if 0 < self.get_mill_count(
                    position.ring_index,
                    position.field_index,
                    DirectionToCheck::OnAndAcrossRings {
                        player_color: (!color).into(),
                    },
                ) {
                    stones_to_take_counter -= 1;
                }
            }

            if (color_positions[0].ring_index == color_positions[1].ring_index
                && color_positions[1].ring_index != color_positions[2].ring_index)
                || (color_positions[0].ring_index != color_positions[1].ring_index
                    && color_positions[1].ring_index == color_positions[2].ring_index)
                || (color_positions[0].ring_index == color_positions[2].ring_index
                    && color_positions[0].ring_index != color_positions[1].ring_index)
                || (color_positions[0].field_index == color_positions[1].field_index
                    && color_positions[1].field_index != color_positions[2].field_index)
                || (color_positions[0].field_index != color_positions[1].field_index
                    && color_positions[1].field_index == color_positions[2].field_index)
                || (color_positions[0].field_index == color_positions[2].field_index
                    && color_positions[0].field_index != color_positions[1].field_index)
            {
                moves_into_mill_counter += 1;
            }
        } else {
            self.calculate_move_tupel(
                color,
                &mut moves_possible_counter,
                &mut moves_into_mill_counter,
                &mut overall_stones_of_opposite_color_counter,
                &mut stones_to_take_counter,
            );

            if stones_to_take_counter == 0 {
                // All stones of the opposite color are in a mill:
                stones_to_take_counter = overall_stones_of_opposite_color_counter;
            }
        }

        (moves_possible_counter, moves_into_mill_counter, stones_to_take_counter)
    }

    fn calculate_move_tupel(
        &mut self,
        color: PlayerColor,
        moves_possible_counter: &mut u32,
        moves_into_mill_counter: &mut u32,
        overall_stones_of_opposite_color_counter: &mut u32,
        stones_to_take_counter: &mut u32,
    ) {
        for ring_index in 0..3 {
            for field_index in 0..8 {
                // Current field state sifted to the LSB
                let current_field_state = self.get_field_state_at(ring_index, field_index, true);

                // If the current field is empty, we wont make any adjustments to the return values
                if current_field_state == 0 {
                    continue;
                }

                // If the current field == color && the on-ring neighbors are empty => movements into a mill possible
                if current_field_state == color.into() {
                    for (neighbor_index, neighbor_state) in self.get_neighbor_field_states(ring_index, field_index) {
                        if neighbor_state == 0 {
                            *moves_possible_counter += 1;

                            *moves_into_mill_counter += self.simulate_move_get_mill_count(
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
                    if (field_index % 2) == 0 {
                        let (a, b) = self.calculate_tupels_across_rings(ring_index, field_index, color);

                        *moves_possible_counter += a;
                        *moves_into_mill_counter += b;
                    }
                }
                // The opposite colors amount of stones which can be taken should be counted, which is if the stone
                // Isn't inside a mill!
                else {
                    *overall_stones_of_opposite_color_counter += 1;

                    if self.get_mill_count(
                        ring_index,
                        field_index,
                        DirectionToCheck::OnAndAcrossRings {
                            player_color: current_field_state,
                        },
                    ) == 0
                    {
                        *stones_to_take_counter += 1;
                    }
                }
            }
        }
    }

    /// Calculates the moves possible and moves into a mill when a stones is movable across EfficientPlayField rings.
    /// Returns (moves_possible_counter, moves_into_mill_counter)
    fn calculate_tupels_across_rings(&mut self, ring_index: usize, field_index: u16, color: PlayerColor) -> (u32, u32) {
        let mut moves_possible_counter = 0;
        let mut moves_into_mill_counter = 0;

        let next_rings_field_state =
            self.get_field_state_at(Self::get_ring_index(ring_index, Next), field_index, false);
        let previous_rings_field_state =
            self.get_field_state_at(Self::get_ring_index(ring_index, Previous), field_index, false);

        match ring_index {
            // Inner Ring
            0 if next_rings_field_state == 0 => {
                moves_possible_counter += 1;

                moves_into_mill_counter += self.simulate_move_get_mill_count(
                    0,
                    field_index,
                    MoveDirection::AcrossRings { target_ring_index: 1 },
                    color.into(),
                )
            }
            // Mid Ring
            1 => {
                if previous_rings_field_state == 0 {
                    moves_possible_counter += 1;

                    moves_into_mill_counter += self.simulate_move_get_mill_count(
                        1,
                        field_index,
                        MoveDirection::AcrossRings { target_ring_index: 0 },
                        color.into(),
                    )
                }

                if next_rings_field_state == 0 {
                    moves_possible_counter += 1;

                    moves_into_mill_counter += self.simulate_move_get_mill_count(
                        1,
                        field_index,
                        MoveDirection::AcrossRings { target_ring_index: 2 },
                        color.into(),
                    )
                }
            }
            // Outer Ring
            2 if previous_rings_field_state == 0 => {
                moves_possible_counter += 1;

                moves_into_mill_counter += self.simulate_move_get_mill_count(
                    2,
                    field_index,
                    MoveDirection::AcrossRings { target_ring_index: 1 },
                    color.into(),
                )
            }
            _ => {}
        }

        return (moves_possible_counter, moves_into_mill_counter);
    }

    fn get_positions(&self, color: PlayerColor) -> (SmallVec<[FieldPos; 9]>, SmallVec<[FieldPos; 9]>) {
        let mut color_positions = SmallVec::<[FieldPos; 9]>::default();
        let not_color_positions = SmallVec::<[FieldPos; 9]>::default();

        for ring_index in 0..3 {
            for field_index in 0..8 {
                let state = self.get_field_state_at(ring_index, field_index, true);
                if state == color.into() {
                    color_positions.push(FieldPos {
                        ring_index,
                        field_index,
                    })
                } else if state == (!color).into() {
                    color_positions.push(FieldPos {
                        ring_index,
                        field_index,
                    })
                }
            }
        }

        (color_positions, not_color_positions)
    }

    /// Simulates a move of the stones of the start fields and ring index to either a it's neighboring target index or
    /// the start index on another ring, which is determined by the [MoveDirection] enum.
    ///
    /// Preconditions:
    /// - Indices should already be in "abstract form" x < 8
    /// - The target field/ the start index on the other ring must be empty
    // TODO test if out-of-place performs better here
    fn simulate_move_get_mill_count(
        &mut self,
        start_ring_index: usize,
        start_fields_index: u16,
        direction: MoveDirection,
        color: u16,
    ) -> u32 {
        assert!(start_fields_index < 8);

        // To rollback the in-situ changes on self
        let start_ring_backup = self.state[start_ring_index];

        // Clear out the current index, must be done when simulating the moving in general
        self.state[start_ring_index] &= !(3u16 << (start_fields_index * 2));

        let mills_possible = if let MoveDirection::AcrossRings { target_ring_index } = direction {
            // To rollback the second in-situ changes on self
            let target_ring_backup = self.state[target_ring_index];

            // Setting the state of the other index, which must be empty
            self.state[target_ring_index] |= color << (start_fields_index * 2);

            // TODO makes this sense to you, future me? :|
            let mills_possible = self.get_mill_count(target_ring_index, start_fields_index, DirectionToCheck::OnRing);
            //let mills_possible = self.get_mill_count(target_ring_index, start_fields_index, DirectionToCheck::OnAndAcrossRings { player_color: color });

            // Resetting the in-place simulation on the other ring
            self.state[target_ring_index] = target_ring_backup;

            mills_possible
        } else if let MoveDirection::OnRing { target_field_index } = direction {
            assert!(target_field_index < 8);

            // Set the empty neighbors value to the old one of the current index:
            self.state[start_ring_index] |= color << (target_field_index * 2);

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
    /// - The fields index must be 0..8 and the ring index 0..3
    fn get_mill_count(&self, ring_index: usize, field_index: u16, direction: DirectionToCheck) -> u32 {
        assert!(field_index < 8);
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
        let rep_indices_to_rotate = (((field_index - (field_index % 2) + 7) % 8) * 2) as u32;
        // Field state triple containing field_index:
        let state_triple = self.state[ring_index].rotate_right(rep_indices_to_rotate) & 0b0000_0000_0011_1111u16;

        /* 010101 | 101010 */
        if state_triple == 21u16 || state_triple == 42u16 {
            mill_counter += 1;
        }

        // If index is located in an edge, two triples must be checked for mill occurrence
        if field_index == 1 || field_index == 3 || field_index == 5 || field_index == 7 {
            let state_triple = self.state[ring_index].rotate_right((field_index * 2) as u32) & 0b000_00000_0011_1111u16;
            /* 010101 | 101010 */
            if state_triple == 21u16 || state_triple == 42u16 {
                mill_counter += 1;
            }
        }

        // Argument field index in the middle of a triple and therefore can form a mill connected to the other rings
        if let DirectionToCheck::OnAndAcrossRings { player_color } = direction {
            if field_index % 2 == 0 {
                //assert!(((self.state[ring_index] >> field_index) & 3u16) != 0);
                let next_rindex_field_state =
                    self.get_field_state_at(Self::get_ring_index(ring_index, Next), field_index, true);

                let previous_rindex_field_state =
                    self.get_field_state_at(Self::get_ring_index(ring_index, Previous), field_index, true);

                // Mill in between rings:
                if next_rindex_field_state == player_color && next_rindex_field_state == previous_rindex_field_state {
                    mill_counter += 1;
                }
            }
        }
        mill_counter
    }
}

/// Used by the [simulate_move_then_get_mills] method of [EfficientPlayField]
pub enum MoveDirection {
    OnRing { target_field_index: u16 },
    AcrossRings { target_ring_index: usize },
}

/// Used by the [get_mill_count] method of [EfficientPlayField]
enum DirectionToCheck {
    OnRing,
    OnAndAcrossRings { player_color: u16 },
}

fn process_input_fields_canonical() {
    let (reader, mut writer) = init_writer_reader("input_felder_4.txt");
    let mut output_map = FnvHashMap::<EfficientPlayField, usize>::default();

    for (line_index, line_content) in reader.lines().enumerate() {
        // Idk why but the reference output.txt starts counting on 1...
        let line_index = line_index + 1;

        let mut playfield = EfficientPlayField::from_coded(&line_content.unwrap());
        println!("{playfield}");
        let canonical_form = playfield.get_canonical_form();
        println!("{canonical_form}");

        match output_map.get(&canonical_form) {
            Some(identical_canonical_index) => writeln!(writer, "{}", identical_canonical_index).unwrap(),
            None => {
                output_map.insert(canonical_form, line_index);
                writeln!(writer, "{}", line_index).unwrap();
            }
        }
    }
}

fn process_input_fields_tuple() {
    let (reader, mut writer) = init_writer_reader("input_felder_5.txt");

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

/// Inits the reader and writer on the default files `input_felder.txt` and `output.txt`
fn init_writer_reader(input: &str) -> (BufReader<File>, BufWriter<File>) {
    let input_felder_txt =
        File::open(input).expect("The 'input_felder.txt' file was not found in the projects root...");
    let reader = BufReader::new(input_felder_txt);

    let output_text = File::create("output.txt").expect("Could not create ro 'output.txt' to write results into");
    let writer = BufWriter::new(output_text);

    return (reader, writer);
}

#[cfg(test)]
mod tests {
    use super::EfficientPlayField;

    #[test]
    fn assignment4() {
        super::process_input_fields_canonical();
    }

    #[test]
    fn assignment5() {
        super::process_input_fields_tuple();
    }

    #[test]
    fn assignment5_dbg() {
        //let mut test_epf = EfficientPlayField::from_coded("BEEEWEWBEEWWEEWEWEEWWWBB");
        let mut test_epf = EfficientPlayField::from_coded("EWWBBEEEEEWBBEEEEEEEBEEB");
        println!("{test_epf}");

        let (x, y, z) = test_epf.get_move_triple(crate::game::PlayerColor::White);
        println!("{x} {y} {z}")
    }

    mod normal {
        use super::*;

        #[test]
        fn set_field_normal() {
            let mut epf = EfficientPlayField::default();

            epf.set_field_state(2, 7, 2); // ring 2, index 7, to black
            epf.set_field_state(1, 7, 1); // ring 1, index 7, to white
            epf.set_field_state(0, 7, 1); // ring 0, index 7, to white

            epf.set_field_state(1, 0, 2); // ring 1, index 0, to black
            epf.set_field_state(1, 1, 2); // ring 1, index 1, to black
            epf.set_field_state(1, 2, 2); // ring 1, index 2, to black
            epf.set_field_state(1, 3, 2); // ring 1, index 3, to black
            epf.set_field_state(1, 4, 2); // ring 1, index 4, to black
            epf.set_field_state(1, 5, 2); // ring 1, index 5, to black
            epf.set_field_state(1, 6, 2); // ring 1, index 6, to black

            epf.set_field_state(0, 6, 2); // ring 1, index 6, to black
            epf.set_field_state(2, 2, 2); // ring 1, index 6, to black
            epf.set_field_state(2, 4, 1); // ring 1, index 6, to black

            println!("\nAfter some added stones: {}", epf);
        }

        #[test]
        fn rotate1() {
            let mut epf = EfficientPlayField::default();

            epf.set_field_state(2, 0, 1);
            epf.set_field_state(1, 1, 2);
            epf.set_field_state(0, 2, 1);

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

            epf.set_field_state(2, 0, 1);
            epf.set_field_state(1, 1, 2);
            epf.set_field_state(0, 2, 1);
            epf.set_field_state(2, 3, 2);

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

            epf.set_field_state(2, 7, 2); // ring 2, index 7, to black
            epf.set_field_state(2, 7, 2); // ring 2, index 7, to white
        }

        #[test]
        #[should_panic]
        fn set_field_white_to_black() {
            let mut epf = EfficientPlayField::default();

            epf.set_field_state(2, 7, 1); // ring 2, index 7, to white
            epf.set_field_state(2, 7, 2); // ring 2, index 7, to black
        }

        #[test]
        #[should_panic]
        fn set_field_free_to_free() {
            let mut epf = EfficientPlayField::default();

            epf.set_field_state(1, 3, 0); // ring 1, index 3, to white
        }

        #[test]
        #[should_panic]
        fn set_field_to_11() {
            let mut epf = EfficientPlayField::default();

            epf.set_field_state(1, 3, 4); // ring 1, index 3, to undefined
        }

        #[test]
        #[should_panic]
        fn set_ring_index_to_11() {
            let mut epf = EfficientPlayField::default();

            epf.set_field_state(3, 1, 2); // ring ?, index 1, to black
        }
    }
}
