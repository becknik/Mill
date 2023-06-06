use std::collections::HashSet;

use super::DirectionToCheck;
use super::EfficientPlayField;
use super::MoveDirection;
use crate::game::PlayerColor;

impl EfficientPlayField {
    /// Returns the bit masks for the fields that can be taken by the player with player_color
    fn get_fields_to_take(&self, player_color: PlayerColor) -> Vec<(usize, u16)> {
        let mut all_stone_bitmasks = Vec::<(usize, u16)>::new();
        let mut not_in_mill_bitsmasks = Vec::<(usize, u16)>::new();

        for ring_index in 0..3 {
            for field_index in (0..16).step_by(2) {
                let current_field_state = (self.state[ring_index] & (3u16 << field_index)) >> field_index;

                if current_field_state == 0 || current_field_state == <PlayerColor as Into<u16>>::into(!player_color) {
                    continue;
                }

                let bit_mask = 3u16 << field_index;
                all_stone_bitmasks.push((ring_index, bit_mask));

                if 0 == self.get_mill_count(
                    ring_index,
                    field_index,
                    DirectionToCheck::OnAndAcrossRings {
                        player_color: player_color.into(),
                    },
                ) {
                    not_in_mill_bitsmasks.push((ring_index, bit_mask));
                }
            }
        }

        // If all stones are in mills, stones from mills can be taken
        if not_in_mill_bitsmasks.is_empty() {
            all_stone_bitmasks
        } else {
            not_in_mill_bitsmasks
        }
    }

    /// Preconditions:
    /// - Indices should already be in "representation form" (= 0 <= x < 16).step_by(2)
    /// - The target field/ the start index on the other ring must be empty
    fn simulate_moves(
        &mut self,
        fields_to_take: &Vec<(usize, u16)>,
        start_ring_index: usize,
        start_fields_index: u16,
        direction: MoveDirection,
        color: u16,
    ) -> Vec<EfficientPlayField> {
        let mut simulated_playfields = Vec::<EfficientPlayField>::new();

        // To rollback the in-situ changes on self
        let start_ring_backup = self.state[start_ring_index];

        // Clear out the current index, must be done when simulating the moving in general
        self.state[start_ring_index] &= !(3u16 << start_fields_index);

        if let MoveDirection::AcrossRings { target_ring_index } = direction {
            // To rollback the second in-situ changes on self
            let target_ring_backup = self.state[target_ring_index];

            // Setting the state of the other index, which must be empty
            self.state[target_ring_index] |= color << start_fields_index;

            let mills_possible = self.get_mill_count(target_ring_index, start_fields_index, DirectionToCheck::OnRing);

            // If mill ocurrs, take all stones possible which is specified in fields_to_take
            if 0 < mills_possible {
                let backup_after_first_move = self.state;

                for field_and_bitmask in fields_to_take {
                    self.state[field_and_bitmask.0] &= !field_and_bitmask.1;
                    simulated_playfields.push(self.clone());

                    self.state = backup_after_first_move;
                }
            }
            // No mills => just push simulated move
            else {
                simulated_playfields.push(self.clone());
            }

            // Resetting the in-place simulation on the other ring
            self.state[target_ring_index] = target_ring_backup;
        } else if let MoveDirection::OnRing { target_field_index } = direction {
            // Set the empty neighbors value to the old one of the current index:
            self.state[start_ring_index] |= color << target_field_index;

            // Check for mills after the move now has taken place
            let mills_possible = self.get_mill_count(
                start_ring_index,
                target_field_index,
                DirectionToCheck::OnAndAcrossRings { player_color: color },
            );

            // See commentary above
            if 0 < mills_possible {
                let backup_after_first_move = self.state;

                for field_and_bitmask in fields_to_take {
                    self.state[field_and_bitmask.0] &= !(field_and_bitmask.1);
                    simulated_playfields.push(self.clone());

                    self.state = backup_after_first_move;
                }
            } else {
                simulated_playfields.push(self.clone());
            }
        }

        // Resetting the in-place simulation
        self.state[start_ring_index] = start_ring_backup;

        return simulated_playfields;
    }

    pub fn generate_forward_moves(&mut self, player_color: PlayerColor) -> Vec<EfficientPlayField> {
        let fields_to_take = self.get_fields_to_take(!player_color);

        let mut forward_moved_playfields = Vec::<EfficientPlayField>::new();

        for ring_index in 0..3 {
            for field_index in (0..16).step_by(2) {
                let current_field_state = (self.state[ring_index] & (3u16 << field_index)) >> field_index;
                if current_field_state == 0 {
                    continue;
                }

                if current_field_state == <PlayerColor as Into<u16>>::into(player_color) {
                    let neighbors_indices_on_ring = [(field_index + 14) % 16, (field_index + 18) % 16];
                    for neighbor_index in neighbors_indices_on_ring {

                        if (self.state[ring_index] & (3u16 << neighbor_index)) == 0 {
                            let mut current_move_playfields = self.simulate_moves(
                                &fields_to_take,
                                ring_index,
                                field_index,
                                MoveDirection::OnRing {
                                    target_field_index: neighbor_index,
                                },
                                current_field_state,
                            );
                            forward_moved_playfields.append(&mut current_move_playfields);
                        }
                    }

                    if (field_index % 4) == 0 {
                        let next_rings_field_state = self.state[(ring_index + 1) % 3] & (3u16 << field_index);
                        let previous_rings_field_state = self.state[(ring_index + 2) % 3] & (3u16 << field_index);

                        match ring_index {
                            0 if next_rings_field_state == 0 => {
                                let mut current_move_playfields = self.simulate_moves(
                                    &fields_to_take,
                                    0,
                                    field_index,
                                    MoveDirection::AcrossRings { target_ring_index: 1 },
                                    current_field_state,
                                );
                                forward_moved_playfields.append(&mut current_move_playfields);
                            }
                            1 => {
                                if previous_rings_field_state == 0 {
                                    let mut current_move_playfields = self.simulate_moves(
                                        &fields_to_take,
                                        1,
                                        field_index,
                                        MoveDirection::AcrossRings { target_ring_index: 0 },
                                        current_field_state,
                                    );
                                    forward_moved_playfields.append(&mut current_move_playfields);
                                }

                                if next_rings_field_state == 0 {
                                    let mut current_move_playfields = self.simulate_moves(
                                        &fields_to_take,
                                        1,
                                        field_index,
                                        MoveDirection::AcrossRings { target_ring_index: 2 },
                                        current_field_state,
                                    );
                                    forward_moved_playfields.append(&mut current_move_playfields);
                                }
                            }
                            2 if previous_rings_field_state == 0 => {
                                let mut current_move_playfields = self.simulate_moves(
                                    &fields_to_take,
                                    2,
                                    field_index,
                                    MoveDirection::AcrossRings { target_ring_index: 1 },
                                    current_field_state,
                                );
                                forward_moved_playfields.append(&mut current_move_playfields);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        forward_moved_playfields
    }

    fn get_fields_to_place(&self, player_color: PlayerColor) -> Vec<(usize, u16)> {
        let mut empty_fields_to_place_bitmasks = Vec::<(usize, u16)>::new();

        for ring_index in 0..3 {
            for field_index in (0..16).step_by(2) {
                let current_field_state = (self.state[ring_index] & (3u16 << field_index)) >> field_index;

                if current_field_state != 0 {
                    continue;
                }

                let future_field_state: u16 = <PlayerColor as Into<u16>>::into(player_color) << field_index;
                empty_fields_to_place_bitmasks.push((ring_index, future_field_state));
            }
        }

        empty_fields_to_place_bitmasks
    }

    fn simulate_backward_move_get_playfields(
        &mut self,
        fields_to_place: &Vec<(usize, u16)>,
        start_ring_index: usize,
        start_fields_index: u16,
        direction: MoveDirection,
        player_color: PlayerColor,
    ) -> Vec<EfficientPlayField> {
        let mut simulated_playfields = Vec::<EfficientPlayField>::new();

        //set color of stones to place
        let stone_color = player_color.into();

        // To rollback the in-situ changes on self
        let start_ring_backup = self.state[start_ring_index];

        // Check for mills before the move has taken place
        let was_in_mill = self.get_mill_count(
            start_ring_index,
            start_fields_index,
            DirectionToCheck::OnAndAcrossRings {
                player_color: stone_color,
            },
        );

        // Clear out the current index, must be done when simulating the moving in general
        self.state[start_ring_index] &= !(3u16 << start_fields_index);

        if let MoveDirection::AcrossRings { target_ring_index } = direction {
            // To rollback the second in-situ changes on self
            let target_ring_backup = self.state[target_ring_index];

            // Setting the state of the other index, which must be empty
            self.state[target_ring_index] |= stone_color << start_fields_index;

            if 0 < was_in_mill {
                let backup_after_first_move = self.state;

                'outer: for field_and_bitmask in fields_to_place {
                    // excludes field where stone moved to
                    if target_ring_index == field_and_bitmask.0 {
                        for i in 0..8 {
                            if field_and_bitmask.1 & (3u16 << (2 * i)) != 0 {
                                if self.state[target_ring_index] & (0x0003 << (2 * i)) != 0 {
                                    continue 'outer;
                                }
                            }
                        }
                    }

                    self.state[field_and_bitmask.0] |= field_and_bitmask.1;
                    simulated_playfields.push(self.clone());

                    self.state = backup_after_first_move;
                }
            } else {
                simulated_playfields.push(self.clone());
            }

            // Resetting the in-place simulation on the other ring
            self.state[target_ring_index] = target_ring_backup;
        } else if let MoveDirection::OnRing { target_field_index } = direction {
            // Set the empty neighbors value to the old one of the current index:
            self.state[start_ring_index] |= stone_color << target_field_index;

            if 0 < was_in_mill {
                let backup_after_first_move = self.state;

                'outer: for field_and_bitmask in fields_to_place {
                    let target_field_state = self.state[start_ring_index] & (0x0003 << target_field_index);

                    // excludes field where stone moved to
                    if start_ring_index == field_and_bitmask.0 {
                        for i in 0..8 {
                            if field_and_bitmask.1 & (3u16 << (2 * i)) != 0 {
                                if self.state[start_ring_index] & (0x0003 << (2 * i)) != 0 {
                                    continue 'outer;
                                }
                            }
                        }
                    }

                    self.state[field_and_bitmask.0] |= field_and_bitmask.1;
                    simulated_playfields.push(self.clone());

                    self.state = backup_after_first_move;
                }
            } else {
                simulated_playfields.push(self.clone());
            }
        }

        // Resetting the in-place simulation
        self.state[start_ring_index] = start_ring_backup;

        return simulated_playfields;
    }

    /*
    Rückwartszüge:

    Für alle bewegbaren Steine:
        Stein in Mühle?
            bewegen den stein (move)
            Dann stein andere Farbe platzieren auf allen möglichen freien stellen, ursprünglichen Feld
            stein dort in Mühle?
                Alle anderen nicht in Mühle?
                    Platziere
                Sonst
                    Platziere nicht
            Sonst
                Platziere
    */

    /// Simulates the backward moves of player with color player_color by calling [get_fields_to_place]
    pub fn get_backward_moves(&mut self, player_color: PlayerColor) -> Vec<EfficientPlayField> {
        let mut output_playfields = Vec::<EfficientPlayField>::new();

        //current fields to place a stone on, current field excluded
        let mut fields_to_place = self.get_fields_to_place(!player_color);

        for ring_index in 0..3 {
            for field_index in (0..16).step_by(2) {
                // Current field state sifted to the LSB
                let current_field_state = (self.state[ring_index] & (3u16 << field_index)) >> field_index;

                let current_tupel = (
                    ring_index,
                    <PlayerColor as Into<u16>>::into(player_color) << field_index,
                );
                let maybe_index = fields_to_place.iter().position(|tup| *tup == current_tupel);

                if let Some(index) = maybe_index {
                    fields_to_place.remove(index);
                }

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
                            let mut current_move_playfields = self.simulate_backward_move_get_playfields(
                                &fields_to_place,
                                ring_index,
                                field_index,
                                MoveDirection::OnRing {
                                    target_field_index: neighbor_index,
                                },
                                player_color,
                            );
                            output_playfields.append(&mut current_move_playfields);
                        }
                    }

                    // Check for possible over-ring moves
                    if (field_index % 4) == 0 {
                        let next_rings_field_state = self.state[(ring_index + 1) % 3] & (3u16 << field_index);
                        let previous_rings_field_state = self.state[(ring_index + 2) % 3] & (3u16 << field_index);

                        match ring_index {
                            // Inner Ring
                            0 if next_rings_field_state == 0 => {
                                let mut current_move_playfields = self.simulate_backward_move_get_playfields(
                                    &fields_to_place,
                                    0,
                                    field_index,
                                    MoveDirection::AcrossRings { target_ring_index: 1 },
                                    player_color,
                                );
                                output_playfields.append(&mut current_move_playfields);
                            }
                            // Mid Ring
                            1 => {
                                if previous_rings_field_state == 0 {
                                    let mut current_move_playfields = self.simulate_backward_move_get_playfields(
                                        &fields_to_place,
                                        1,
                                        field_index,
                                        MoveDirection::AcrossRings { target_ring_index: 0 },
                                        player_color,
                                    );
                                    output_playfields.append(&mut current_move_playfields);
                                }

                                if next_rings_field_state == 0 {
                                    let mut current_move_playfields = self.simulate_backward_move_get_playfields(
                                        &fields_to_place,
                                        1,
                                        field_index,
                                        MoveDirection::AcrossRings { target_ring_index: 2 },
                                        player_color,
                                    );
                                    output_playfields.append(&mut current_move_playfields);
                                }
                            }
                            // Outer Ring
                            2 if previous_rings_field_state == 0 => {
                                let mut current_move_playfields = self.simulate_backward_move_get_playfields(
                                    &fields_to_place,
                                    2,
                                    field_index,
                                    MoveDirection::AcrossRings { target_ring_index: 1 },
                                    player_color,
                                );
                                output_playfields.append(&mut current_move_playfields);
                            }
                            _ => {}
                        }
                    }
                }
                fields_to_place.push(current_tupel);
            }
        }
        output_playfields
    }

    //unnecessary but works
    fn generate_permutations(prefix: u16, s_count: usize, w_count: usize, permutations: &mut Vec<u16>) {
        if s_count == 0 && w_count == 0 {
            permutations.push(prefix);
            return;
        }

        if s_count > 0 {
            let new_prefix = (prefix << 2) | 0x0002;
            Self::generate_permutations(new_prefix, s_count - 1, w_count, permutations);
        }

        if w_count > 0 {
            let new_prefix = (prefix << 2) | 0x0001;
            Self::generate_permutations(new_prefix, s_count, w_count - 1, permutations);
        }
    }

    /// Hard-coded generation of the only 3 unique (using mirroring, rotation, swapping of ring index) mill positions
    fn generate_muehle_placements() -> Vec<EfficientPlayField> {
        let mut muehle_placement_playfield = Vec::<EfficientPlayField>::new();

        let mut template_1 = EfficientPlayField::default();
        template_1.set_field(2, 7, 1);
        template_1.set_field(2, 0, 1);
        template_1.set_field(2, 1, 1);
        muehle_placement_playfield.push(template_1);

        let mut template_2 = EfficientPlayField::default();
        template_2.set_field(1, 7, 1);
        template_2.set_field(1, 0, 1);
        template_2.set_field(1, 1, 1);
        muehle_placement_playfield.push(template_2);

        let mut template_3 = EfficientPlayField::default();
        template_3.set_field(2, 0, 1);
        template_3.set_field(1, 0, 1);
        template_3.set_field(0, 0, 1);
        muehle_placement_playfield.push(template_3);

        muehle_placement_playfield
    }

    /* fn old_generate_ended_game_plafields() -> HashSet<EfficientPlayField> {
           let mut won_plafields = HashSet::<EfficientPlayField>::new();

           //generate the initial won/lost playfields
           let mut permutations = Vec::<u16>::new();

           for w_count in 0..=6 {
               let s_count = 2;
               Self::generate_permutations(0, s_count, w_count, &mut permutations);
           }

           //all possible placements of the muehle
           let muehle_placement_playfields = Self::generate_muehle_placement_playfields();

           let mut template_counter = 0;
           for template_field in muehle_placement_playfields {
               template_counter += 1;

               for permutation in permutations {

                   let mut current_permutation = permutation;
                   let mut current_playfield = template_field;

                   //insert current permutation into the template field
                   for ring_index in 0..3 {
                       for field_index in 0..8 {

                           if current_permutation == 0 {
                               break; //kann ich auch aus beiden for schleifen raus breaken
                           }
                           current_permutation = current_permutation >> 2;

                           let current_element = 0x0003 & permutation;
                           let current_field_state = template_field.state[ring_index] & (0x0003 << (field_index*2));

                           if current_field_state != 0 {
                               continue;
                           }

                           current_playfield.state[ring_index] &= current_element << (field_index*2);
                       }
                   }

                   //shift permutation through playfield and insert each resulting playfield into output hashset



               }
           }
           won_plafields
       }
    */

    fn generate_playfields_recursive(
        &mut self, // Playfield, initalized with playfields containg a white mill
        mut ring_index: usize,
        mut field_index: u16,
        black_stones_count: usize,
        white_stones_count: usize,
        empty_stones_count: usize,
        playfields: &mut HashSet<EfficientPlayField>,
    ) {
        //println!("Recursive Call!");

        // insert if everything has been placed
        if black_stones_count == 0 && white_stones_count == 0 && empty_stones_count == 0 {
            playfields.insert(self.clone().get_canonical_form());
            return;
        }

        // for-loop index bound simulation
        if field_index == 8 {
            field_index = 0;
            ring_index += 1;
        }

        // if current field is not empty
        if (self.state[ring_index] & (0x0003 << (field_index * 2))) != 0 {
            self.clone().generate_playfields_recursive(
                ring_index,
                field_index + 1,
                black_stones_count,
                white_stones_count,
                empty_stones_count,
                playfields,
            )
        }

        if empty_stones_count > 0 {
            self.clone().generate_playfields_recursive(
                ring_index,
                field_index + 1,
                black_stones_count,
                white_stones_count,
                empty_stones_count - 1,
                playfields,
            )
        }

        if black_stones_count > 0 {
            let mut clone = self.clone();
            clone.state[ring_index] |= 0x0002 << (field_index * 2);
            clone.generate_playfields_recursive(
                ring_index,
                field_index + 1,
                black_stones_count - 1,
                white_stones_count,
                empty_stones_count,
                playfields,
            )
        }

        if white_stones_count > 0 {
            let mut clone = self.clone();
            clone.state[ring_index] |= 0x0001 << (field_index * 2);
            clone.generate_playfields_recursive(
                ring_index,
                field_index + 1,
                black_stones_count,
                white_stones_count - 1,
                empty_stones_count,
                playfields,
            )
        }
    }

    /*

    - stones_of_winning_color between 3 and 9
     */
    pub fn generate_ended_game_plafields(amount_white_stones: usize) -> HashSet<EfficientPlayField> {
        let mut won_plafields = HashSet::<EfficientPlayField>::new();

        for white_stones_count_not_mills /* 0..=6 */ in 0..=(amount_white_stones - 3) {
            let black_stones_count = 2;
            let empty_stones_count = 19 - white_stones_count_not_mills; // 24 - 2 - 3 (=muehle) - rest of amount_white_stones
            let ring_index = 0;
            let field_index = 0;

            //all possible placements of the muehle
            let muehle_placement_playfields = Self::generate_muehle_placements();

            for mut template_field in muehle_placement_playfields {
                template_field.generate_playfields_recursive(
                    ring_index,
                    field_index,
                    black_stones_count,
                    white_stones_count_not_mills,
                    empty_stones_count,
                    &mut won_plafields,
                )
            }
        }

        won_plafields
    }

    pub fn invert_playfields_stone_colors(&self) -> EfficientPlayField {
        let mut current_playfield = self.clone();

        for ring_index in 0..3 {
            for field_index in 0..8 {
                match (current_playfield.state[ring_index] & (0x0003 << (field_index * 2))) >> (field_index * 2) {
                    0x0000 => (),
                    0x0001 => {
                        current_playfield.state[ring_index] = (current_playfield.state[ring_index]
                            & !(0x0003 << (field_index * 2)))
                            | (0x0002 << (field_index * 2))
                    }
                    0x0002 => {
                        current_playfield.state[ring_index] = (current_playfield.state[ring_index]
                            & !(0x0003 << (field_index * 2)))
                            | (0x0001 << (field_index * 2))
                    }
                    _ => {}
                }
            }
        }

        current_playfield
    }

    fn generate_white_won_configurations(canonical_form: bool) -> HashSet<EfficientPlayField> {
        //println!("Test");
        let mut won_set = HashSet::<EfficientPlayField>::new();

        let configs_with_white_mill = Self::generate_muehle_placements();

        for config in configs_with_white_mill {
            //println!("{config}");

            for i in 0..24 {
                let ring_index = (i / 8) as usize;
                let field_index = i % 8;

                // to avoid placing stones onto already present mills
                if (config.state[ring_index] & (3u16 << (field_index * 2))) != 0 {
                    continue;
                }

                let mut config = config.clone();
                config.state[ring_index] |= 0x0002 << (field_index * 2);
                //println!("{clone}");

                for j in (i + 1)..24 {
                    let ring_index = (j / 8) as usize;
                    let field_index = j % 8;

                    // to avoid placing stones onto already present mills
                    if (config.state[ring_index] & (3u16 << (field_index * 2))) != 0 {
                        continue;
                    }

                    let mut config = config.clone();
                    config.state[ring_index] |= 0x0002 << (field_index * 2);
                    //println!("{clone}");

                    won_set.insert(if canonical_form {
                        config.get_canonical_form()
                    } else {
                        config
                    });

                    config.place_stones_across_playfield(PlayerColor::White, 6, 0, &mut won_set, canonical_form);
                }
            }
        }

        //Self::add_white_won_configurations_enclosed_to(&mut won_set, canonical_form);

        won_set
    }

    fn place_stones_across_playfield(
        &self,
        stone_color: PlayerColor,
        recursion_depth: u32,
        start_index: u32,
        set: &mut HashSet<EfficientPlayField>,
        canonical_form: bool,
    ) {
        for i in start_index..24 {
            let ring_index = (i / 8) as usize;
            let field_index = i % 8;

            if (self.state[ring_index] & (3u16 << (field_index * 2))) != 0 {
                continue;
            }

            // TODO use the in-place mutable version here for more preformance
            //let ring_backup = self.state[ring_index];
            let mut modified_self = match ring_index {
                0 => EfficientPlayField {
                    state: [
                        self.state[ring_index] | (<PlayerColor as Into<u16>>::into(stone_color) << (field_index * 2)),
                        self.state[ring_index + 1],
                        self.state[ring_index + 2],
                    ],
                },
                1 => EfficientPlayField {
                    state: [
                        self.state[ring_index - 1],
                        self.state[ring_index] | (<PlayerColor as Into<u16>>::into(stone_color) << (field_index * 2)),
                        self.state[ring_index + 1],
                    ],
                },
                2 => EfficientPlayField {
                    state: [
                        self.state[ring_index - 2],
                        self.state[ring_index - 1],
                        self.state[ring_index] | (<PlayerColor as Into<u16>>::into(stone_color) << (field_index * 2)),
                    ],
                },
                _ => EfficientPlayField::default(), // neglected panic!
            };

            set.insert(if canonical_form {
                modified_self.get_canonical_form()
            } else {
                modified_self
            });

            if 24 <= start_index {
                return;
            } else if 1 < recursion_depth {
                modified_self.place_stones_across_playfield(
                    stone_color,
                    recursion_depth - 1,
                    i + 1,
                    set,
                    canonical_form,
                );
            }

            //self.state[ring_index] = ring_backup;
        }
    }

    // schwarz iterative random, aähnlich so wie in den rekusiven AUfruf oben
    // pro stein, alle züge mit weiß abdecken
    //   wenn 9 weiße setine zu platzieren sind aber noch schwarz oder mehr weiße steine benötigt werden -> continue
    // wenn alles blockiert und übrige weiß > 0 -> random übrige schwarze und weiße platzieren in empty fields
    fn add_white_won_configurations_enclosed_to(set: &mut HashSet<EfficientPlayField>, canonical_form: bool) {
        let pf = EfficientPlayField::default();
        let mut black_only = HashSet::<EfficientPlayField>::new();
        let mut won_set_enclosed = HashSet::<EfficientPlayField>::new();

        for i in 0..24 {
            let ring_index = (i / 8) as usize;
            let field_index = i % 8;

            let mut pf = pf.clone();
            pf.state[ring_index] |= 2u16 << (field_index * 2);

            for j in (i + 1)..24 {
                let ring_index = (j / 8) as usize;
                let field_index = j % 8;

                let mut pf = pf.clone();
                pf.state[ring_index] |= 2u16 << (field_index * 2);

                for k in (j + 1)..24 {
                    let ring_index = (k / 8) as usize;
                    let field_index = k % 8;

                    let mut pf = pf.clone();
                    pf.state[ring_index] |= 2u16 << (field_index * 2);

                    for l in (k + 1)..24 {
                        let ring_index = (l / 8) as usize;
                        let field_index = l % 8;

                        let mut pf = pf.clone();
                        pf.state[ring_index] |= 2u16 << (field_index * 2);

                        black_only.insert(if canonical_form { pf.get_canonical_form() } else { pf });

						// Adding combinations of 4<= playfieds to the black only set
                        // 4 <= due to 3 can't be enclosed by white stones because of possible jumping
                        pf.place_stones_across_playfield(PlayerColor::Black, 7, 0, &mut black_only, canonical_form);
                    }
                }
            }
        }

        //black_only.iter();
    }

    fn enclose_if_possible(&mut self, set: &mut HashSet<EfficientPlayField>) -> bool {
        for ring_index in 0..3 {
            for field_index in 0..8 {}
        }

        false
    }

    //ab hier wirds schwammig
    /*     pub fn generate_won_and_lost_playfields() -> (HashSet<EfficientPlayField>, HashSet<EfficientPlayField>) {
        let mut won_set = HashSet::<EfficientPlayField>::new();
        let mut lost_set = HashSet::<EfficientPlayField>::new();

        let won_ended_set = Self::generate_ended_game_plafields();

        for current_playfield in won_ended_set {
            current_playfield.mark_won(won_set, lost_set);
        }

        (won_set, lost_set)
    }

    fn mark_lost(&self, mut won_set: HashSet<EfficientPlayField>, mut lost_set: HashSet<EfficientPlayField>) {
        if won_set.insert(self.clone()) {
            for current_playfield in self.get_backward_moves(PlayerColor::White) {
                current_playfield.mark_won(won_set, lost_set);
            }
        }
    }

    fn mark_won(&mut self, mut won_set: HashSet<EfficientPlayField>, lost_set: HashSet<EfficientPlayField>) {
        if won_set.insert(self.clone()) {
            for mut current_playfield in self.get_backward_moves(PlayerColor::White) {
                let mut check_var = 0;

                for current_forward in current_playfield.get_forward_moves(PlayerColor::Black) {
                    if won_set.get(&current_forward) == None {
                        check_var += 1;
                        break;
                    }
                }

                if check_var == 0 {
                    current_playfield.mark_lost(won_set, lost_set);
                }
            }
        }
    } */

    /*
    mark_lost(P)
    if (P !∈ L)
        L:= L ∪ {P}         //Globale Variable
        for z ∈ rückwärtsZüge(P)
            P' = z(P)
            mark_won(P')

    mark_won(P)
        if (P !∈ W)
            W:= W ∪ {P}         //Globale Variable
            for z ∈ rückwärtsZüge(P)
                P' = z(P)
                if z'(P') ∈ W für alle z' ∈ vorwärtsZüge(P')
                    mark_lost(P')
    */
}

#[cfg(test)]
mod tests {
    use crate::game::{efficient_state::EfficientPlayField, PlayerColor};

    fn get_tests_set() -> Vec<&'static str> {
        vec!["WWEEEEEWEEEEEEEEEEEEEEEE"]
    }

    #[test]
    fn test_invert_playfield() {
        let test_string = "WWWWBBBBEEEEWWWWBBBBEEEE";
        let test_playfield = EfficientPlayField::from_coded(test_string);

        println!("{test_playfield}");
        println!("{}", test_playfield.invert_playfields_stone_colors());
    }

    #[test]
    fn test_get_fields_to_take() {
        let test_string = "WEEEBEEEWEEEBEEEWEEEBBWB";
        let test_playfield = EfficientPlayField::from_coded(test_string);

        println!("\n--- Initial Playfield ---\n");
        println!("{test_playfield}");
        println!("\n--- Fields with legal stones taken ---\n");

        let vec = test_playfield.get_fields_to_take(PlayerColor::Black);

        let mut i = 0;
        vec.iter()
            .map(|tuple| {
                let mut new_pf = test_playfield.clone();
                new_pf.state[tuple.0] &= !tuple.1;
                new_pf
            })
            .for_each(|pf| {
                println!("> PlayField on Index {i}:\n{pf}");
                i += 1;
            });
    }

    #[test]
    fn test_get_fields_to_place() {
        // WWEEEEEWEEEEEEEEEEEEEEEE
        // WWWWBBBBEEEEWWWWBBBBEEEE

        let test_string = "WWWWBBBBEEEEWWWWBBBBEEEE";
        let test_playfield = EfficientPlayField::from_coded(test_string);

        println!("\n--- Initial Playfield ---\n");
        println!("{test_playfield}");
        println!("\n--- Fields with legal stones placed ---\n");

        let vec = test_playfield.get_fields_to_place(PlayerColor::White);

        let mut i = 0;
        vec.iter()
            .map(|tuple| {
                let mut new_pf = test_playfield.clone();
                new_pf.state[tuple.0] |= tuple.1;
                new_pf
            })
            .for_each(|pf| {
                println!("> PlayField on Index {i}:\n{pf}");
                i += 1;
            });
    }

    #[test]
    fn test_get_forward_moves() {
        // Default-Move-Pattern:        "WEEEEEEEEWEEWEEEEEEEEEWE"
        // Move-Into-Muehle-Pattern:    "WEWEEEEWEEEEEBEBEBEEEEEE"

        let test_string = "WEWEEEEWEEEEEBEBEBEEEEEE";
        let mut test_playfield = EfficientPlayField::from_coded(test_string);

        println!("\n--- Initial Playfield ---\n");
        println!("{test_playfield}");
        println!("\n--- Fields with simulated moves ---\n");

        let vec = test_playfield.generate_forward_moves(PlayerColor::White);

        let mut i = 0;
        vec.iter().for_each(|pf| {
            println!("> PlayField on Index {i}:\n{pf}");
            i += 1;
        });
    }

    #[test]
    fn test_get_backward_moves() {
        // Default-Move-Pattern:        "WEEEEEEEEWEEWEEEEEEEEEWE"
        // Move-Out-Of-Muehle-Pattern:  "WWEEEEEWEEEEEEEEEEEEEEEE"

        let test_string = "WWEEEEEWEEEEEEEEEEEEEEEE";
        let mut test_playfield = EfficientPlayField::from_coded(test_string);

        println!("\n--- Initial Playfield ---\n");
        println!("{test_playfield}");
        println!("\n--- Fields with simulated moves ---\n");

        let vec = test_playfield.get_backward_moves(PlayerColor::White);

        let mut i = 0;
        vec.iter().for_each(|pf| {
            println!("> PlayField on Index {i}:\n{pf}");
            i += 1;
        });
    }

    #[test]
    fn generate_white_won_configurations_test() {
        let won_set = EfficientPlayField::generate_white_won_configurations(false);
        println!("{}", won_set.len())

        /*let mut i = 0;
        won_set.iter()
            .filter(|pf| {
                let mut white_stones_count = 0;

                for i in 0..24 {
                    let ring_index = (i / 8) as usize;
                    let field_index = i % 8;

                    let current_index_state = pf.state[ring_index] & (3u16 << (field_index * 2));
                    if current_index_state == (1u16 << (field_index * 2)) {
                        white_stones_count += 1;
                    }
                }

                white_stones_count == 5
            })
            .for_each(|pf| {
                println!("> PlayField on Index {i}:\n{pf}");
                i += 1;
            }
        );*/
    }
}
