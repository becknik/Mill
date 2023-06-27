use crate::game::{
    efficient_state::{DirectionToCheck, EfficientPlayField, FieldPos, MoveDirection},
    PlayerColor,
};

mod backward;
mod forward;

impl EfficientPlayField {
    pub fn get_forward_moves(&mut self, player_color: PlayerColor) -> Vec<EfficientPlayField> {
        // Place here because it seems to be used in both branches...
        let fields_to_take = self.get_fields_to_take(!player_color);

        let mut forward_moved_playfields = Vec::<EfficientPlayField>::new();
        let mut simulated_playfield_buffer = Vec::<EfficientPlayField>::new();

        for ring_index in 0..3 {
            for field_index in 0..8 {
                let current_field_state = self.get_field_state_at(ring_index, field_index, true);

                if current_field_state == 0 {
                    continue;
                }

                if current_field_state == player_color.into() {
                    let amount_of_stones = self.get_stone_count_of(player_color);

                    // only 3 stones of current color? -> enable jumping
                    if amount_of_stones == 3 {
                        let backup_state = self.state;

                        //clear current field
                        self.state[ring_index] &= !(0x0003 << (field_index * 2));

                        for empty_field in self.get_empty_fields() {
                            if ring_index == empty_field.ring_index && field_index == empty_field.field_index {
                                continue;
                            }

                            let mut clone = self.clone();

                            clone.state[empty_field.ring_index] |=
                                <PlayerColor as Into<u16>>::into(player_color) << (empty_field.field_index * 2);

                            let mills_possible = clone.get_mill_count(
                                empty_field.ring_index,
                                empty_field.field_index,
                                DirectionToCheck::OnAndAcrossRings {
                                    player_color: player_color.into(),
                                },
                            );

                            // If no mill occurred, just add the new config
                            if 0 == mills_possible {
                                forward_moved_playfields.push(clone);
                            }
                            // If a new mill occurs through jump, simulate the possible takes &
                            // add them to the forward_moved_playfields vec
                            else {
                                let backup_after_first_move = clone.state;

                                for field in &fields_to_take {
                                    clone.state[field.ring_index] &= !(3u16 << (field.field_index * 2));
                                    forward_moved_playfields.push(clone);

                                    clone.state = backup_after_first_move;
                                }
                            }
                        }
                        self.state = backup_state;
                    } else {
                        for (neighbor_index, neighbor_state) in self.get_neighbor_field_states(ring_index, field_index)
                        {
                            if neighbor_state == 0 {
                                self.simulate_possible_forward_moves_for(
                                    &fields_to_take,
                                    FieldPos {
                                        ring_index,
                                        field_index,
                                    },
                                    MoveDirection::OnRing {
                                        target_field_index: neighbor_index,
                                    },
                                    player_color.into(),
                                    &mut simulated_playfield_buffer,
                                );

                                forward_moved_playfields.append(&mut simulated_playfield_buffer);
                                simulated_playfield_buffer.clear();
                            }
                        }

                        if (field_index % 4) == 0 {
                            let next_rings_field_state = self.state[(ring_index + 1) % 3] & (3u16 << field_index);
                            let previous_rings_field_state = self.state[(ring_index + 2) % 3] & (3u16 << field_index);

                            match ring_index {
                                0 if next_rings_field_state == 0 => {
                                    self.simulate_possible_forward_moves_for(
                                        &fields_to_take,
                                        FieldPos {
                                            ring_index: 0,
                                            field_index,
                                        },
                                        MoveDirection::AcrossRings { target_ring_index: 1 },
                                        player_color.into(),
                                        &mut simulated_playfield_buffer,
                                    );
                                    forward_moved_playfields.append(&mut simulated_playfield_buffer);
                                    simulated_playfield_buffer.clear();
                                }
                                1 => {
                                    if previous_rings_field_state == 0 {
                                        self.simulate_possible_forward_moves_for(
                                            &fields_to_take,
                                            FieldPos {
                                                ring_index: 1,
                                                field_index,
                                            },
                                            MoveDirection::AcrossRings { target_ring_index: 0 },
                                            player_color.into(),
                                            &mut simulated_playfield_buffer,
                                        );
                                        forward_moved_playfields.append(&mut simulated_playfield_buffer);
                                        simulated_playfield_buffer.clear();
                                    }

                                    if next_rings_field_state == 0 {
                                        self.simulate_possible_forward_moves_for(
                                            &fields_to_take,
                                            FieldPos {
                                                ring_index: 1,
                                                field_index,
                                            },
                                            MoveDirection::AcrossRings { target_ring_index: 2 },
                                            player_color.into(),
                                            &mut simulated_playfield_buffer,
                                        );
                                        forward_moved_playfields.append(&mut simulated_playfield_buffer);
                                        simulated_playfield_buffer.clear();
                                    }
                                }
                                2 if previous_rings_field_state == 0 => {
                                    self.simulate_possible_forward_moves_for(
                                        &fields_to_take,
                                        FieldPos {
                                            ring_index: 2,
                                            field_index,
                                        },
                                        MoveDirection::AcrossRings { target_ring_index: 1 },
                                        player_color.into(),
                                        &mut simulated_playfield_buffer,
                                    );
                                    forward_moved_playfields.append(&mut simulated_playfield_buffer);
                                    simulated_playfield_buffer.clear();
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        forward_moved_playfields
    }

    /// Simulates the backward moves of player with color player_color by calling [get_fields_to_place]
    pub fn get_backward_moves(&mut self, player_color: PlayerColor) -> Vec<EfficientPlayField> {
        let mut output_playfields = Vec::<EfficientPlayField>::new();

        //current fields to place a stone on, current field excluded
        let mut fields_to_place = self.get_empty_field_bitmasks(!player_color);

        for ring_index in 0..3 {
            for field_index in (0..16).step_by(2) {
                // Current field state sifted to the LSB
                let current_field_state: u16 = (self.state[ring_index] & (3u16 << field_index)) >> field_index;

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
                    let amount_of_stones = self.get_stone_count_of(player_color);
                    if amount_of_stones == 3 {
                        // Check for mills before the move has taken place
                        let was_in_mill = self.get_mill_count(
                            ring_index,
                            field_index / 2,
                            DirectionToCheck::OnAndAcrossRings {
                                player_color: current_field_state,
                            },
                        );

                        // Add all jump configurations into the vec
                        let fields_to_place = self.get_empty_field_bitmasks(player_color);

                        let backup_state = self.state;
                        self.state[ring_index] &= !(0x0003 << field_index);

                        for placement in fields_to_place {
                            let mut clone = self.clone();

                            clone.state[placement.0] |= placement.1;

                            if 0 < was_in_mill {
                                let fields_to_place_taken_stone = clone.get_empty_field_bitmasks(!player_color);

                                for replacement in fields_to_place_taken_stone {
                                    if !(ring_index == replacement.0 && replacement.1 & (0x0003 << field_index) != 0) {
                                        let mut clone_2 = clone.clone();

                                        clone_2.state[replacement.0] |= replacement.1;

                                        //TODO
                                        /* let mut stones_not_in_mill = 0;
                                        for ring_index in 0..3 {
                                            for field_index in 0..8 {
                                                if 0 == clone_2.get_mill_count(ring_index, field_index, DirectionToCheck::OnAndAcrossRings { player_color: (!player_color).into()}) {
                                                    stones_not_in_mill += 1;
                                                }
                                            }
                                        }

                                        let mut new_field_index = 0;

                                        while replacement.1 != 0 {
                                            replacement.1 = replacement.1 >> 2;
                                            new_field_index += 1;
                                        }
                                        new_field_index -= 1;

                                        let mills_possible = clone_2.get_mill_count(
                                            replacement.0,
                                            new_field_index,
                                            DirectionToCheck::OnAndAcrossRings {
                                                player_color: (!player_color).into(),
                                            },
                                        );

                                        if stones_not_in_mill > 0 {
                                            if mills_possible == 0 {
                                                output_playfields.push(clone_2.clone());
                                            }
                                        } else {
                                            output_playfields.push(clone_2.clone());
                                        } */

                                        //wenn alle steine in mühle -> push
                                        //wenn nicht
                                        // wenn dieser in mühle -> nicht push
                                        // wenn nicht -> push

                                        /* if clone_2.get_mill_count(replacement.0, replacement.1, DirectionToCheck::OnAndAcrossRings { player_color: !player_color.into() }) {
                                            //ganz viele mühle checks:()
                                        } */

                                        output_playfields.push(clone_2.clone());
                                    }
                                }
                            } else {
                                output_playfields.push(clone.clone());
                            }

                            /* if 0 < was_in_mill {
                                let backup_after_first_move = clone.state;

                                for field_and_bitmask in &fields_to_take {
                                    clone.state[field_and_bitmask.0] &= !field_and_bitmask.1;
                                    forward_moved_playfields.push(clone.clone());

                                    clone.state = backup_after_first_move;
                                }
                            } else {
                                forward_moved_playfields.insert(0, clone);
                            } */
                        }
                        self.state = backup_state;
                    } else {
                        let ring_neighbors_indices = [(field_index + 14) % 16, (field_index + 18) % 16];
                        for neighbor_index in ring_neighbors_indices {
                            // Neighbor field state is empty - neighbor_index already are representational index (0 <= i < 16)
                            if (self.state[ring_index] & (3u16 << neighbor_index)) == 0 {
                                let mut current_move_playfields = self.simulate_backward_move_get_playfields(
                                    &fields_to_place,
                                    ring_index,
                                    field_index, // hier geteilt 2
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
                }
                fields_to_place.push(current_tupel);
            }
        }
        output_playfields
    }
}
