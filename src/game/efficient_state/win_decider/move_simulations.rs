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
                            if empty_field
                                == (FieldPos {
                                    ring_index,
                                    field_index,
                                })
                            {
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

    #[rustfmt::skip]
    /// Simulates the backward moves of player with color player_color by calling [get_fields_to_place]
    pub fn get_backward_moves(&mut self, player_color: PlayerColor) -> Vec<EfficientPlayField> {
        let empty_fields = self.get_empty_fields();

        let mut output_playfields = Vec::<EfficientPlayField>::new();
        let mut simulated_playfield_buffer = Vec::<EfficientPlayField>::new();

        for ring_index in 0..3 {
            for field_index in 0..8 {
                let current_field_state = self.get_field_state_at(ring_index, field_index, true);

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
                            field_index,
                            DirectionToCheck::OnAndAcrossRings {
                                player_color: current_field_state,
                            },
                        );

                        let backup_state = self.state;
                        // clear out the current position
                        self.state[ring_index] &= !(0x0003 << (field_index * 2));

                        // make jump-moves onto all free positions
                        for to_jump_field in &empty_fields {
                            // skip the current field
                            /* if field == (FieldPos{ring_index, field_index}) {
                                continue;
                            } */

                            let mut clone = self.clone();

                            // Apply the jump to the state clone
                            clone.state[to_jump_field.ring_index] |=
                                <PlayerColor as Into<u16>>::into(player_color) << (to_jump_field.field_index * 2);

                            if 0 == was_in_mill {
                                output_playfields.push(clone);
                            }
                            // If the jump was made by a stone which was previously located in a mill,
                            // stones from the other color, which were previously taken by the color with the mill,
                            // have to be added to the field again
                            else {
                                // self.add_simulated_placements(start, player_color, simulated_playfields); TODO!!!
                                for to_place_field in clone.get_empty_fields() {
                                    let mut after_jump_clone = clone.clone();

                                    // sorts out initial mill position
                                    if to_place_field == (FieldPos { ring_index, field_index, }) {
                                        continue;
                                    }

                                    // adds opposite colored stone (which has been taken be the mill) to empty field
                                    after_jump_clone.state[to_place_field.ring_index] |= <PlayerColor as Into<u16>>::into(!player_color) << (to_place_field.field_index * 2);

                                    // if the placed stone of the opposite color could be taken now,
                                    // the placement of this stone would be valid and the current playfield config should be pushed
                                    if after_jump_clone.get_fields_to_take(player_color).contains(&to_place_field) {
                                        output_playfields.push(after_jump_clone);
                                    }

                                    // TODO what is this? and remove
                                    /* if !(ring_index == replacement.0 && replacement.1 & (0x0003 << field_index) != 0) {
                                        let mut clone_2 = clone.clone();

                                        clone_2.state[replacement.0] |= replacement.1;

                                        //TODO
                                        let mut stones_not_in_mill = 0;
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
                                        }

                                        //wenn alle steine in mühle -> push
                                        //wenn nicht
                                        // wenn dieser in mühle -> nicht push
                                        // wenn nicht -> push

                                        if clone_2.get_mill_count(replacement.0, replacement.1, DirectionToCheck::OnAndAcrossRings { player_color: !player_color.into() }) {
                                            //ganz viele mühle checks:()
                                        }

                                        output_playfields.push(clone_2.clone());
                                    } */
                                }
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
                        for (neighbor_index, neighbor_state) in self.get_neighbor_field_states(ring_index, field_index) {

                            if (self.state[ring_index] & (3u16 << neighbor_index)) == 0 {
                                self.simulate_backward_move_get_playfields(
                                    &empty_fields,
                                    FieldPos {ring_index, field_index },
                                    MoveDirection::OnRing {
                                        target_field_index: neighbor_index,
                                    },
                                    player_color,
                                    &mut simulated_playfield_buffer
                                );
                                output_playfields.append(&mut simulated_playfield_buffer);
                                simulated_playfield_buffer.clear();
                            }
                        }

                        // Check for possible over-ring moves
                        if (field_index % 4) == 0 {
                            let next_rings_field_state = self.state[(ring_index + 1) % 3] & (3u16 << field_index);
                            let previous_rings_field_state = self.state[(ring_index + 2) % 3] & (3u16 << field_index);

                            match ring_index {
                                // Inner Ring
                                0 if next_rings_field_state == 0 => {
                                    self.simulate_backward_move_get_playfields(
                                        &empty_fields,
                                        FieldPos {ring_index: 0, field_index },
                                        MoveDirection::AcrossRings { target_ring_index: 1 },
                                        player_color,
                                        &mut simulated_playfield_buffer,
                                    );
                                    output_playfields.append(&mut simulated_playfield_buffer);
                                    simulated_playfield_buffer.clear();
                                }
                                // Mid Ring
                                1 => {
                                    if previous_rings_field_state == 0 {
                                        self.simulate_backward_move_get_playfields(
                                            &empty_fields,
                                            FieldPos {ring_index: 1, field_index },
                                            MoveDirection::AcrossRings { target_ring_index: 0 },
                                            player_color,
                                            &mut simulated_playfield_buffer
                                        );
                                        output_playfields.append(&mut simulated_playfield_buffer);
                                        simulated_playfield_buffer.clear();
                                    }

                                    if next_rings_field_state == 0 {
                                        self.simulate_backward_move_get_playfields(
                                            &empty_fields,
                                            FieldPos {ring_index: 1, field_index },
                                            MoveDirection::AcrossRings { target_ring_index: 2 },
                                            player_color,
                                            &mut simulated_playfield_buffer
                                        );
                                        output_playfields.append(&mut simulated_playfield_buffer);
                                        simulated_playfield_buffer.clear();
                                    }
                                }
                                // Outer Ring
                                2 if previous_rings_field_state == 0 => {
                                    self.simulate_backward_move_get_playfields(
                                        &empty_fields,
                                        FieldPos {ring_index: 2, field_index },
                                        MoveDirection::AcrossRings { target_ring_index: 1 },
                                        player_color,
                                        &mut simulated_playfield_buffer
                                    );
                                    output_playfields.append(&mut simulated_playfield_buffer);
                                    simulated_playfield_buffer.clear();
                                }
                                _ => {}
                            }
                        }
                    }
                }
                // empty_fields.push(current_tupel); // TODO wtf?! this makes no sense
            }
        }
        output_playfields
    }

    fn add_simulated_placements(
        &mut self,
        start: FieldPos,
        player_color: PlayerColor,
        simulated_playfields: &mut Vec<EfficientPlayField>,
    ) {
        let backup_after_move = self.state;

        for to_place_field in self.get_empty_fields() {
            // sorts out initial mill position
            if to_place_field
                == (FieldPos {
                    ring_index: start.ring_index,
                    field_index: start.field_index,
                })
            {
                continue;
            }

            // adds opposite colored stone (which has been taken be the mill) to empty field
            self.state[to_place_field.ring_index] |=
                <PlayerColor as Into<u16>>::into(!player_color) << (to_place_field.field_index * 2);

            // if the placed stone of the opposite color could be taken now,
            // the placement of this stone would be valid and the current playfield config should be pushed
            if self.get_fields_to_take(player_color).contains(&to_place_field) {
                simulated_playfields.push(self.clone());
            }

            self.state = backup_after_move;
        }
    }
}
