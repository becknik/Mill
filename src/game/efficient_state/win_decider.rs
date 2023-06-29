use std::{
    collections::HashSet,
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

use smallvec::SmallVec;

use super::EfficientPlayField;
use super::MoveDirection;
use super::{DirectionToCheck, FieldPos};
use crate::game::PlayerColor;

mod move_simulations;
mod start_set_generation;

const TO_TAKE_VEC_SIZE: usize = 64;

impl EfficientPlayField {
    /// Counts & returns the amount of stones on the whole plazfield
    fn get_stone_count_of(&self, player_color: PlayerColor) -> u32 {
        let mut stone_counter = 0;

        for ring_index in 0..3 {
            for field_index in 0..8 {
                let current_field_state = self.get_field_state_at(ring_index, field_index, true);

                if current_field_state == player_color.into() {
                    stone_counter += 1;
                }
            }
        }
        stone_counter
    }

    /// Returns the FieldPos field coordinates of stones that can be taken by the player with player_color
    /// Therefore, the SmallVec returns only fields with stones of the color !player_color
    fn get_fields_to_take_by(&self, player_color: PlayerColor) -> SmallVec<[FieldPos; TO_TAKE_VEC_SIZE]> {
        let mut all_stones_to_take_pos = SmallVec::<[FieldPos; TO_TAKE_VEC_SIZE]>::new();
        let mut not_in_mill_pos = SmallVec::<[FieldPos; TO_TAKE_VEC_SIZE]>::new();

        let opponent_player_color_rep: u16 = (!player_color).into();

        for ring_index in 0..3 {
            for field_index in 0..8 {
                let current_field_state = self.get_field_state_at(ring_index, field_index, true);

                if current_field_state == opponent_player_color_rep {
                    all_stones_to_take_pos.push(FieldPos { ring_index, field_index });

                    // If the opponent has no mill on this field, add this field to the appropriate set
                    if 0 == self.get_mill_count(
                        ring_index,
                        field_index,
                        DirectionToCheck::OnAndAcrossRings { player_color: opponent_player_color_rep },
                    ) {
                        not_in_mill_pos.push(FieldPos { ring_index, field_index });
                    }
                }
            }
        }

        // If all stones are in mills, stones from mills can be taken
        if not_in_mill_pos.is_empty() {
            all_stones_to_take_pos
        } else {
            not_in_mill_pos
        }
    }

    /// Returns the fields which are free to place a stone upon.
    fn get_empty_fields(&self) -> SmallVec<[FieldPos; 19]> {
        let mut empty_fields = SmallVec::<[FieldPos; 19]>::new();

        for ring_index in 0..3 {
            for field_index in 0..8 {
                let current_field_state = self.get_field_state_at(ring_index, field_index, false);

                if current_field_state == 0 {
                    empty_fields.push(FieldPos { ring_index, field_index });
                }
            }
        }
        empty_fields
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

    // schwarz iterative random, aähnlich so wie in den rekusiven AUfruf oben
    // pro stein, alle züge mit weiß abdecken
    //   wenn 9 weiße setine zu platzieren sind aber noch schwarz oder mehr weiße steine benötigt werden -> continue
    // wenn alles blockiert und übrige weiß > 0 -> random übrige schwarze und weiße platzieren in empty fields
    fn add_white_won_configurations_enclosed(max_stone_count: usize, won_set: &mut FnvHashSet<EfficientPlayField>) {
        let pf = EfficientPlayField::default();
        let mut black_only = FnvHashSet::<EfficientPlayField>::new();
        //let mut won_set_enclosed = HashSet::<EfficientPlayField>::new();

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

                        black_only.insert(pf.get_canonical_form());

                        // Adding combinations of 4<= playfieds to the black only set
                        // 4 <= due to 3 can't be enclosed by white stones because of possible jumping
                        pf.place_stones_across_playfield(
                            PlayerColor::Black,
                            (max_stone_count as i32 - 4).max(0) as usize,
                            l + 1,
                            &mut black_only,
                        );
                    }
                }
            }
        }

        for mut playfield in black_only {
            playfield.enclose_if_possible(max_stone_count, won_set);
        }
    }

    // Returns self with added white stones that enclose black stones,
    // and if possible extra placements of left over white stones
    fn enclose_if_possible(&mut self, max_stone_count: usize, set: &mut HashSet<EfficientPlayField>) {
        let white_enclosing_moves = self.get_forward_move_placements();
        let amount_of_white_moves = white_enclosing_moves.len(); // neccessary beacuase of move

        // if there are less unique placements than 9: place white stones upon those fields to block moves
        // 9 - black_mill_count:  there are some black mills on the playfield, the amount of white placed stone
        // previously was reduced by the number of black mills
        if amount_of_white_moves <= max_stone_count {
            // places a white stone on all possible placements
            for (ring_index, bitmask_field_index) in white_enclosing_moves {
                self.state[ring_index] |= bitmask_field_index;
            }

            // insert playfield with the enclosure without extra stones placed
            set.insert(self.clone().get_canonical_form());

            // if there are leftovers, all possible placements are done and added to the set
            let left_overs = (max_stone_count as i32 - amount_of_white_moves as i32) as usize;

            self.place_stones_across_playfield(PlayerColor::White, left_overs, 0, set);
        }

        /* let white_enclosing_moves = self.get_forward_move_placements();
        let amount_of_white_moves = white_enclosing_moves.len(); // neccessary beacuase of move

        let (black_mill_count, crossed_mill_exist) =
            self.get_total_amount_of_mills_and_double_mills(PlayerColor::Black);
        let black_mill_count = black_mill_count - crossed_mill_exist; // TODO this might be wrong

        // if there are less unique placements than 9: place white stones upon those fields to block moves
        // 9 - black_mill_count:  there are some black mills on the playfield, the amount of white placed stone
        // previously was reduced by the number of black mills
        if amount_of_white_moves <= 0.max(max_stone_count as i32 - black_mill_count as i32) as usize {
            // places a white stone on all possible placements
            for (ring_index, bitmask_field_index) in white_enclosing_moves {
                self.state[ring_index] |= bitmask_field_index;
            }

            // insert playfield with the enclosure without extra stones placed
            set.insert(self.clone().get_canonical_form());

            // if there are leftovers, all possible placements are done and added to the set
            let left_overs =
                0.max(max_stone_count as i32 - amount_of_white_moves as i32 - black_mill_count as i32) as usize;

            self.place_stones_across_playfield(PlayerColor::White, left_overs, 0, set);
        } */
    }

    // Returns amount of mills present of one color on the playfields
    fn get_total_amount_of_mills_and_double_mills(&self, color: PlayerColor) -> (usize, usize) {
        let mut mill_count: usize = 0;
        let mut double_mill_count: usize = 0;

        let mut lane_stone_count = [0; 4];
        for ring_index in 0..3 {
            for field_index in 0..8 {
                if field_index % 2 == 0 {
                    //hier
                    mill_count += self.get_mill_count(ring_index, field_index, DirectionToCheck::OnRing) as usize;

                    let current_even_index_state = (self.state[ring_index] << (field_index * 2)) >> (field_index * 2);

                    if current_even_index_state == color.into() {
                        lane_stone_count[(field_index / 2) as usize] += 1;
                    }
                }

                // TODO passdas?
                if self.get_mill_count(
                    ring_index,
                    field_index,
                    DirectionToCheck::OnAndAcrossRings { player_color: color.into() },
                ) == 2
                {
                    double_mill_count += 1;
                }
            }
        }

        for elem in lane_stone_count {
            if elem == 3 {
                mill_count += 1;
            }
        }

        return (mill_count, double_mill_count);
    }

    // Returns a Set containing ring_index
    // and a mask containing the white stone at the right placement field for the enclosure for one stone
    fn get_move_placements(
        &mut self,
        start_ring_index: usize,
        start_fields_index: u16,
        direction: MoveDirection,
    ) -> HashSet<(usize, u16)> {
        let mut move_placements = HashSet::<(usize, u16)>::new();

        if let MoveDirection::AcrossRings { target_ring_index } = direction {
            let placement_mask = 0x0001 << start_fields_index;

            move_placements.insert((target_ring_index, placement_mask));
        } else if let MoveDirection::OnRing { target_field_index } = direction {
            let placement_mask = 0x0001 << target_field_index;

            move_placements.insert((start_ring_index, placement_mask));
        }

        return move_placements;
    }

    // Returns all placement_masks with the correct placement of the white stones for the enclosure
    pub fn get_forward_move_placements(&mut self) -> HashSet<(usize, u16)> {
        let mut output_placements = HashSet::<(usize, u16)>::new();

        for ring_index in 0..3 {
            for field_index in (0..16).step_by(2) {
                let current_field_state = self.state[ring_index] & (3u16 << field_index);

                if current_field_state == 0 {
                    continue;
                }

                // All possible enclose placements are added into the Set
                let ring_neighbors_indices = [(field_index + 14) % 16, (field_index + 18) % 16];
                for neighbor_index in ring_neighbors_indices {
                    // Neighbor field state is empty - neighbor_index already are representational index (0 <= i < 16)
                    if (self.state[ring_index] & (3u16 << neighbor_index)) == 0 {
                        let current_move_placements = self.get_move_placements(
                            ring_index,
                            field_index,
                            MoveDirection::OnRing { target_field_index: neighbor_index },
                        );
                        output_placements.extend(current_move_placements);
                    }
                }

                // Check for possible over-ring moves
                if (field_index % 4) == 0 {
                    let next_rings_field_state = self.state[(ring_index + 1) % 3] & (3u16 << field_index);
                    let previous_rings_field_state = self.state[(ring_index + 2) % 3] & (3u16 << field_index);

                    match ring_index {
                        // Inner Ring
                        0 if next_rings_field_state == 0 => {
                            let current_move_placements = self.get_move_placements(
                                0,
                                field_index,
                                MoveDirection::AcrossRings { target_ring_index: 1 },
                            );
                            output_placements.extend(current_move_placements);
                        }
                        // Mid Ring
                        1 => {
                            if previous_rings_field_state == 0 {
                                let current_move_placements = self.get_move_placements(
                                    1,
                                    field_index,
                                    MoveDirection::AcrossRings { target_ring_index: 0 },
                                );
                                output_placements.extend(current_move_placements);
                            }

                            if next_rings_field_state == 0 {
                                let current_move_placements = self.get_move_placements(
                                    1,
                                    field_index,
                                    MoveDirection::AcrossRings { target_ring_index: 2 },
                                );
                                output_placements.extend(current_move_placements);
                            }
                        }
                        // Outer Ring
                        2 if previous_rings_field_state == 0 => {
                            let current_move_placements = self.get_move_placements(
                                2,
                                field_index,
                                MoveDirection::AcrossRings { target_ring_index: 1 },
                            );
                            output_placements.extend(current_move_placements);
                        }
                        _ => {}
                    }
                }
            }
        }
        output_placements
    }

    pub fn generate_all_won_playfields(
        max_stone_count: usize,
    ) -> (HashSet<EfficientPlayField>, HashSet<EfficientPlayField>) {
        let mut won_set = EfficientPlayField::generate_white_won_configurations(max_stone_count);
        EfficientPlayField::add_white_won_configurations_enclosed(max_stone_count, &mut won_set);
        println!("> Created WON set containing {} elements", won_set.len());

        let mut work_queue = VecDeque::<(usize, EfficientPlayField)>::new();

        for pf in &won_set {
            // TODO change this to the maximum depth when it is known
            work_queue.push_back((0, *pf));
        }
        println!("> Pushed WON sets elements onto queue");

        // generates lost_set for white
        let mut lost_set = HashSet::<EfficientPlayField>::new();

        // Indicator for who moved last: even => white made last move
        //let mut path_depth: usize = 0;

        let mut counter = 0;
        while let Some((tree_level_bottom_up, mut current)) = work_queue.pop_front() {
            counter += 1;
            if counter % 1_000 == 0 {
                println!(
                    "Bottom up niveau: {tree_level_bottom_up}\nWON length: {} --- LOST length: {}\nQueue length: {}",
                    won_set.len(),
                    lost_set.len(),
                    work_queue.len()
                );
            }

            // White moved last
            if tree_level_bottom_up % 2 == 0 {
                // Every backward move is going to be added:
                for mut backward_move_config in current.get_backward_moves(PlayerColor::White) {
                    backward_move_config = backward_move_config.get_canonical_form();

                    if !won_set.contains(&backward_move_config) {
                        won_set.insert(backward_move_config);
                        work_queue.push_back((tree_level_bottom_up + 1, backward_move_config));
                    }
                }
            }
            //Black moved last
            else {
                for mut backward_playfield in current.get_backward_moves(PlayerColor::Black) {
                    let mut all_forward_moves_in_won = true;
                    backward_playfield = backward_playfield.get_canonical_form();

                    for mut forward_playfield in backward_playfield.get_forward_moves(PlayerColor::Black) {
                        forward_playfield = forward_playfield.get_canonical_form();
                        if !won_set.contains(&forward_playfield) {
                            all_forward_moves_in_won = false;
                        }
                    }

                    // Adds the inverted backward_playfield to lost_set
                    if all_forward_moves_in_won {
                        let insert_playfield = backward_playfield.invert_playfields_stone_colors().get_canonical_form();

                        if !lost_set.contains(&insert_playfield) {
                            lost_set.insert(insert_playfield);
                            work_queue.push_back((tree_level_bottom_up + 1, backward_playfield));
                        }
                    }
                }
            }
        }

        (won_set, lost_set)
    }

    pub fn input_game_state_decider(max_stone_count: usize) {
        let input_felder_txt = File::open("input_felder_3.txt")
            .expect("The 'input_felder.txt' file was not found in the projects root...");
        let reader = BufReader::new(input_felder_txt);

        let output_text = File::create("output.txt").expect("Could not create ro 'output.txt' to write results into");
        let mut writer = BufWriter::new(output_text);

        let (won_set, lost_set) = EfficientPlayField::generate_all_won_playfields(max_stone_count);
        println!("> Finished generating all sets:");
        println!("> Won: {} --- Lost: {}", won_set.len(), lost_set.len());

        for line_content in reader.lines() {
            let mut playfield = EfficientPlayField::from_coded(&line_content.unwrap());
            let canonical_form = playfield.get_canonical_form();

            let nash_value = if won_set.contains(&canonical_form) {
                2
            } else if lost_set.contains(&canonical_form) {
                0
            } else {
                1
            };

            writeln!(writer, "{}", nash_value).unwrap();
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod unit_tests;
