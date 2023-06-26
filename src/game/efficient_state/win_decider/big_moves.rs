use crate::game::{
    efficient_state::{DirectionToCheck, EfficientPlayField, FieldPos, MoveDirection},
    PlayerColor,
};

impl EfficientPlayField {
    // TODO Maybe let @Anton refactor this?
    pub fn get_forward_moves(&mut self, player_color: PlayerColor) -> Vec<EfficientPlayField> {
        // 3 mal schleifen wtf, inefficiency 100
        let fields_to_take = self.get_fields_to_take(!player_color);

        let mut forward_moved_playfields = Vec::<EfficientPlayField>::new();
        let mut simulated_playfield_buffer = Vec::<EfficientPlayField>::new();

        for ring_index in 0..3 {
            for field_index in (0..16).step_by(2) {
                let current_field_state = self.get_field_state_at(ring_index, field_index, true);

                if current_field_state == 0 {
                    continue;
                }

                if current_field_state == player_color.into() {
                    // only 3 stones? -> jumps
                    let amount_of_stones = self.get_stone_count_of(player_color);
                    if amount_of_stones == 3 {
                        let backup_state = self.state;
                        self.state[ring_index] &= !(0x0003 << field_index);

                        // Add all jump configurations into the vec
                        let fields_to_place = self.get_empty_field_bitmasks(player_color);
                        for mut placement in fields_to_place {
                            let mut clone = self.clone();

                            clone.state[placement.0] |= placement.1;
                            // Reverse engineering the field index from bitmask
                            let mut new_field_index = 0;

                            while placement.1 != 0 {
                                placement.1 = placement.1 >> 2;
                                new_field_index += 1;
                            }
                            new_field_index -= 1;

                            let mills_possible = clone.get_mill_count(
                                placement.0,
                                new_field_index,
                                DirectionToCheck::OnAndAcrossRings {
                                    player_color: player_color.into(),
                                },
                            );

                            if 0 < mills_possible {
                                let backup_after_first_move = clone.state;

                                for field_and_bitmask in &fields_to_take {
                                    clone.state[field_and_bitmask.0] &= !field_and_bitmask.1;
                                    forward_moved_playfields.push(clone.clone());

                                    clone.state = backup_after_first_move;
                                }
                            } else {
                                forward_moved_playfields.insert(0, clone);
                            }
                        }
                        self.state = backup_state;
                    } else {
                        let neighbors_indices_on_ring = [(field_index + 14) % 16, (field_index + 18) % 16];
                        for neighbor_index in neighbors_indices_on_ring {
                            if (self.state[ring_index] & (3u16 << neighbor_index)) == 0 {
                                self.simulate_all_possible_moves(
                                    &fields_to_take.to_vec(),
                                    FieldPos {
                                        ring_index,
                                        field_index,
                                    },
                                    MoveDirection::OnRing {
                                        target_field_index: neighbor_index,
                                    },
                                    current_field_state,
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
                                    self.simulate_all_possible_moves(
                                        &fields_to_take.to_vec(),
                                        FieldPos {
                                            ring_index: 0,
                                            field_index,
                                        },
                                        MoveDirection::AcrossRings { target_ring_index: 1 },
                                        current_field_state,
                                        &mut simulated_playfield_buffer,
                                    );
                                    forward_moved_playfields.append(&mut simulated_playfield_buffer);
                                    simulated_playfield_buffer.clear();
                                }
                                1 => {
                                    if previous_rings_field_state == 0 {
                                        self.simulate_all_possible_moves(
                                            &fields_to_take.to_vec(),
                                            FieldPos {
                                                ring_index: 1,
                                                field_index,
                                            },
                                            MoveDirection::AcrossRings { target_ring_index: 0 },
                                            current_field_state,
                                            &mut simulated_playfield_buffer,
                                        );
                                        forward_moved_playfields.append(&mut simulated_playfield_buffer);
                                        simulated_playfield_buffer.clear();
                                    }

                                    if next_rings_field_state == 0 {
                                        self.simulate_all_possible_moves(
                                            &fields_to_take.to_vec(),
                                            FieldPos {
                                                ring_index: 1,
                                                field_index,
                                            },
                                            MoveDirection::AcrossRings { target_ring_index: 2 },
                                            current_field_state,
                                            &mut simulated_playfield_buffer,
                                        );
                                        forward_moved_playfields.append(&mut simulated_playfield_buffer);
                                        simulated_playfield_buffer.clear();
                                    }
                                }
                                2 if previous_rings_field_state == 0 => {
                                    self.simulate_all_possible_moves(
                                        &fields_to_take.to_vec(),
                                        FieldPos {
                                            ring_index: 2,
                                            field_index,
                                        },
                                        MoveDirection::AcrossRings { target_ring_index: 1 },
                                        current_field_state,
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
}
