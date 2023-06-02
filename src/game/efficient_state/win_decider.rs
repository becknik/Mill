use std::collections::HashSet;
use std::hash::Hash;
use std::thread::current;

use super::DirectionToCheck;
use super::EfficientPlayField;
use super::MoveDirection;
use crate::game::PlayerColor;

impl EfficientPlayField {
    /// Returns the bit masks for the fields that can be taken
    fn get_fields_to_take(&self, player_color: PlayerColor) -> Vec<(usize, u16)> {
        let mut all_stone_bitmasks = Vec::<(usize, u16)>::new();
        let mut not_in_mill_bitsmasks = Vec::<(usize, u16)>::new();

        for ring_index in 0..3 {
            for field_index in (0..16).step_by(2) {
                let current_field_state = (self.state[ring_index] & (3u16 << field_index)) >> field_index;

                if current_field_state == 0 || current_field_state == <PlayerColor as Into<u16>>::into(!player_color) {
                    continue;
                }

                let bit_mask = 0x0003 << field_index;
                all_stone_bitmasks.push((ring_index, bit_mask));

                if 0 < self.get_mill_count(
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

    /// Calculates the possible moves of player_color, the amount of moves wich lead to a mill for player_color
    /// and the amount of stones of the other player color, which can be beaten
    ///
    /// It is possibly used as a judging function for the player agent
    pub fn get_forward_moves(&mut self, player_color: PlayerColor) -> Vec<EfficientPlayField> {
        let fields_to_take = self.get_fields_to_take(!player_color);

        let mut output_playfields = Vec::<EfficientPlayField>::new();

        for ring_index in 0..3 {
            for field_index in (0..16).step_by(2) {
                // Current field state sifted to the LSB
                let current_field_state = (self.state[ring_index] & (3u16 << field_index)) >> field_index;

                // If the current field is empty, we wont make any adjustments to the return values
                if current_field_state == 0 {
                    continue;
                }

                // In this branch the current colors possible moves & => movements into a mill should be figured out
                if current_field_state == <PlayerColor as Into<u16>>::into(player_color) {
                    let ring_neighbors_indices = [(field_index + 14) % 16, (field_index + 18) % 16];
                    for neighbor_index in ring_neighbors_indices {
                        // Neighbor field state is empty - neighbor_index already are representational index (0 <= i < 16)
                        if (self.state[ring_index] & (3u16 << neighbor_index)) == 0 {
                            let mut current_move_playfields = self.simulate_move_get_playfields(
                                &fields_to_take,
                                ring_index,
                                field_index,
                                MoveDirection::OnRing {
                                    target_field_index: neighbor_index,
                                },
                                current_field_state,
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
                                let mut current_move_playfields = self.simulate_move_get_playfields(
                                    &fields_to_take,
                                    0,
                                    field_index,
                                    MoveDirection::AcrossRings { target_ring_index: 1 },
                                    current_field_state,
                                );
                                output_playfields.append(&mut current_move_playfields);
                            }
                            // Mid Ring
                            1 => {
                                if previous_rings_field_state == 0 {
                                    let mut current_move_playfields = self.simulate_move_get_playfields(
                                        &fields_to_take,
                                        1,
                                        field_index,
                                        MoveDirection::AcrossRings { target_ring_index: 0 },
                                        current_field_state,
                                    );
                                    output_playfields.append(&mut current_move_playfields);
                                }

                                if next_rings_field_state == 0 {
                                    let mut current_move_playfields = self.simulate_move_get_playfields(
                                        &fields_to_take,
                                        1,
                                        field_index,
                                        MoveDirection::AcrossRings { target_ring_index: 2 },
                                        current_field_state,
                                    );
                                    output_playfields.append(&mut current_move_playfields);
                                }
                            }
                            // Outer Ring
                            2 if previous_rings_field_state == 0 => {
                                let mut current_move_playfields = self.simulate_move_get_playfields(
                                    &fields_to_take,
                                    2,
                                    field_index,
                                    MoveDirection::AcrossRings { target_ring_index: 1 },
                                    current_field_state,
                                );
                                output_playfields.append(&mut current_move_playfields);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        output_playfields
    }

    /// Simulates a move of the stones of the start fields and ring index to either a it's neighboring target index or
    /// the start index on another ring, which is determined by the [MoveDirection] enum.
    ///
    /// Preconditions:
    /// - Indices should already be in "representation form" (= 0 <= x < 16).step_by(2)
    /// - The target field/ the start index on the other ring must be empty
    // TODO test if out-of-place performs better here
    fn simulate_move_get_playfields(
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

            if 0 < mills_possible {
                for field_and_bitmask in fields_to_take {
                    self.state[field_and_bitmask.0] &= !field_and_bitmask.1;
                    simulated_playfields.push(self.clone());
                }
            } else {
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

            if 0 < mills_possible {
                for field_and_bitmask in fields_to_take {
                    self.state[field_and_bitmask.0] &= !(field_and_bitmask.1);
                    simulated_playfields.push(self.clone());
                }
            } else {
                simulated_playfields.push(self.clone());
            }
        }

        // Resetting the in-place simulation
        self.state[start_ring_index] = start_ring_backup;

        return simulated_playfields;
    }

    //ab hier
    fn get_fields_to_place(
        &self,
        player_color: PlayerColor,
        ring_index_not: usize,
        field_index_not: u16,
    ) -> Vec<(usize, u16)> {
        let mut empty_fields_bitmasks = Vec::<(usize, u16)>::new();

        for ring_index in 0..3 {
            for field_index in (0..16).step_by(2) {
                let current_field_state = (self.state[ring_index] & (3u16 << field_index)) >> field_index;

                if current_field_state != 0 || (ring_index == ring_index_not && field_index == field_index_not) {
                    continue;
                }

                let bit_mask: u16 = player_color.into();
                empty_fields_bitmasks.push((ring_index, bit_mask << field_index));
            }
        }

        empty_fields_bitmasks
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
                for field_and_bitmask in fields_to_place {
                    self.state[field_and_bitmask.0] &= field_and_bitmask.1;
                    simulated_playfields.push(self.clone());
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
                for field_and_bitmask in fields_to_place {
                    self.state[field_and_bitmask.0] &= !(field_and_bitmask.1);
                    simulated_playfields.push(self.clone());
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

    pub fn get_backward_moves(&mut self, player_color: PlayerColor) -> Vec<EfficientPlayField> {
        let mut output_playfields = Vec::<EfficientPlayField>::new();

        for ring_index in 0..3 {
            for field_index in (0..16).step_by(2) {
                // Current field state sifted to the LSB
                let current_field_state = (self.state[ring_index] & (3u16 << field_index)) >> field_index;

                //current fields to place a stone on, current field excluded
                let current_fields_to_place = self.get_fields_to_place(!player_color, ring_index, field_index);

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
                                &current_fields_to_place,
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
                                    &current_fields_to_place,
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
                                        &current_fields_to_place,
                                        1,
                                        field_index,
                                        MoveDirection::AcrossRings { target_ring_index: 0 },
                                        player_color,
                                    );
                                    output_playfields.append(&mut current_move_playfields);
                                }

                                if next_rings_field_state == 0 {
                                    let mut current_move_playfields = self.simulate_backward_move_get_playfields(
                                        &current_fields_to_place,
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
                                    &current_fields_to_place,
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
            }
        }
        output_playfields
    }

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

    fn generate_muehle_placement_playfields() -> Vec<EfficientPlayField> {
        let mut muehle_placement_playfield = Vec::<EfficientPlayField>::new();

        let mut template_1 = EfficientPlayField::default();
        template_1.set_field(2, 7, 1);
        template_1.set_field(2, 0, 1);
        template_1.set_field(0, 1, 1);
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
    fn generate_playfields_rekursive(
        &mut self,
        mut ring_index: usize,
        mut field_index: u16,
        s_count: usize,
        w_count: usize,
        e_count: usize,
        playfields: &mut HashSet<EfficientPlayField>,
    ) {
        if (s_count == 0 && w_count == 0) && e_count == 0 {
            playfields.insert(self.clone());
            return;
        }

        if field_index == 8 {
            field_index = 0;
            ring_index += 1;
        }

        if self.state[ring_index] & (0x0003 << (field_index * 2)) != 0 {
            self.clone().generate_playfields_rekursive(
                ring_index,
                field_index + 1,
                s_count,
                w_count,
                e_count,
                playfields,
            )
        }

        if e_count > 0 {
            self.clone().generate_playfields_rekursive(
                ring_index,
                field_index + 1,
                s_count,
                w_count,
                e_count - 1,
                playfields,
            )
        }

        if s_count > 0 {
            let mut clone = self.clone();
            clone.state[ring_index] &= 0x0002 << (field_index * 2);
            clone.generate_playfields_rekursive(ring_index, field_index + 1, s_count - 1, w_count, e_count, playfields)
        }

        if w_count > 0 {
            let mut clone = self.clone();
            clone.state[ring_index] &= 0x0001 << (field_index * 2);
            clone.generate_playfields_rekursive(ring_index, field_index + 1, s_count, w_count - 1, e_count, playfields)
        }
    }

    pub fn generate_ended_game_plafields() -> HashSet<EfficientPlayField> {
        let mut won_plafields = HashSet::<EfficientPlayField>::new();

        for w_count in 0..=6 {
            let s_count = 2;
            let e_count = 19 - w_count;
            let ring_index = 0;
            let field_index = 0;

            //all possible placements of the muehle
            let muehle_placement_playfields = Self::generate_muehle_placement_playfields();

            for mut template_field in muehle_placement_playfields {
                template_field.generate_playfields_rekursive(
                    ring_index,
                    field_index,
                    s_count,
                    w_count,
                    e_count,
                    &mut won_plafields,
                )
            }
        }

        won_plafields
    }

    fn as_string(input: u16) -> String {
        let bit_picker: u16 = 0b1100_0000_0000_0000;
        let mut output_string = String::new();

        for i in 0..8 {
            let current_element = (input & (bit_picker >> (2 * i))) >> (14 - (2 * i));
            match current_element {
                0x0000 => (),
                0x0001 => output_string.push('W'),
                0x0002 => output_string.push('B'),
                _ => panic!(),
            }
        }
        output_string
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
                    _ => panic!(),
                }
            }
        }

        current_playfield
    }

    //ab hier wirds schwammig
    pub fn generate_won_and_lost_playfields() -> (HashSet<EfficientPlayField>, HashSet<EfficientPlayField>) {
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
    }

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
    use crate::game::efficient_state::EfficientPlayField;

    #[test]
    fn test_generate_permutations() {
        let mut permutations = Vec::<u16>::new();

        for w_count in 0..=6 {
            let s_count = 2;
            EfficientPlayField::generate_permutations(0, s_count, w_count, &mut permutations);
        }

        for permutation in permutations {
            let string_output = EfficientPlayField::as_string(permutation);
            //println!("{:016b}", permutation);
            println!("{}", string_output);
        }
    }
}
