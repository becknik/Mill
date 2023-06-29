use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

use fnv::FnvHashSet;
use smallvec::SmallVec;

use super::EfficientPlayField;
use super::{DirectionToCheck, FieldPos};
use crate::game::PlayerColor;

mod move_simulations;
mod start_set_generation;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod unit_tests;

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

    //machts hier nicht sinn vllt doch player_color andersrum zu machen?

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

    pub fn generate_won_configs_black_and_white(
        max_stone_count: i32,
    ) -> (FnvHashSet<EfficientPlayField>, FnvHashSet<EfficientPlayField>) {
        let mut won_set = EfficientPlayField::generate_start_won_configs_white(max_stone_count);

        println!("> Finished generation of start WON set containing {} elements", won_set.len());

        let mut work_queue = VecDeque::<(usize, EfficientPlayField)>::new();

        for pf in &won_set {
            // TODO change this to the maximum depth when it is known
            work_queue.push_back((0, *pf));
        }
        println!("> Pushed WON sets elements onto queue");

        // generates lost_set for white
        let mut lost_set = FnvHashSet::<EfficientPlayField>::default();

        // Indicator for who moved last: even => white made last move
        //let mut path_depth: usize = 0;

        let mut counter = 0;
        while let Some((tree_level_bottom_up, mut current)) = work_queue.pop_front() {
            // debug stuff
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
                    backward_move_config = backward_move_config.get_canon_form();

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
                    backward_playfield = backward_playfield.get_canon_form();

                    for mut forward_playfield in backward_playfield.get_forward_moves(PlayerColor::Black) {
                        forward_playfield = forward_playfield.get_canon_form();
                        if !won_set.contains(&forward_playfield) {
                            all_forward_moves_in_won = false;
                        }
                    }

                    // Adds the inverted backward_playfield to lost_set
                    if all_forward_moves_in_won {
                        let insert_playfield = backward_playfield.invert_playfields_stone_colors().get_canon_form();

                        if !lost_set.contains(&insert_playfield) {
                            lost_set.insert(insert_playfield);
                            work_queue.push_back((tree_level_bottom_up + 1, backward_playfield));
                        }
                    }
                }
            }
        }

        (lost_set, won_set)
    }

    pub fn invert_playfields_stone_colors(&self) -> EfficientPlayField {
        let mut current_playfield = self.clone();

        for ring_index in 0..3 {
            for field_index in 0..8 {
                match self.get_field_state_at(ring_index, field_index, true) {
                    1u16 => {
                        current_playfield.state[ring_index] = (current_playfield.state[ring_index]
                            & !(3u16 << (field_index * 2)))
                            | (2u16 << (field_index * 2))
                    }
                    2u16 => {
                        current_playfield.state[ring_index] = (current_playfield.state[ring_index]
                            & !(3u16 << (field_index * 2)))
                            | (1u16 << (field_index * 2))
                    }
                    _ => {}
                }
            }
        }
        current_playfield
    }

    pub fn input_game_state_decider(max_stone_count: i32) {
        let input_felder_txt = File::open("input_felder_3.txt")
            .expect("The 'input_felder.txt' file was not found in the projects root...");
        let reader = BufReader::new(input_felder_txt);

        let output_text = File::create("output.txt").expect("Could not create ro 'output.txt' to write results into");
        let mut writer = BufWriter::new(output_text);

        let (won_set, lost_set) = EfficientPlayField::generate_won_configs_black_and_white(max_stone_count);
        println!("> Finished generating all sets:");
        println!("> Won: {} --- Lost: {}", won_set.len(), lost_set.len());

        for line_content in reader.lines() {
            let mut playfield = EfficientPlayField::from_coded(&line_content.unwrap());
            let canonical_form = playfield.get_canon_form();

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
