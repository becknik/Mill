use smallvec::SmallVec;

use crate::game::{
    efficient_state::{DirectionToCheck, EfficientPlayField, FieldPos, MoveDirection},
    PlayerColor,
};

impl EfficientPlayField {
    // returns vec of one move from one given stone
    pub fn simulate_backward_move_get_playfields(
        &mut self,
        empty_fields: &SmallVec<[FieldPos; 19]>,
        start: FieldPos,
        direction: MoveDirection,
        player_color: PlayerColor,
        simulated_playfields: &mut Vec<EfficientPlayField>,
    ) {
        let stone_color: u16 = player_color.into();

        let start_ring_backup = self.state[start.ring_index];

        let init_mill_count = self.get_mill_count(
            start.ring_index,
            start.field_index,
            DirectionToCheck::OnAndAcrossRings { player_color: stone_color },
        );

        // Clear out the current index
        self.state[start.ring_index] &= !(3u16 << (start.field_index * 2));

        if let MoveDirection::AcrossRings { target_ring_index } = direction {
            let target_ring_backup = self.state[target_ring_index];

            // Setting the moved stone on the other ring
            self.state[target_ring_index] |= stone_color << (start.field_index * 2);

            if init_mill_count == 0 {
                simulated_playfields.push(*self);
            } else {
                self.add_simulated_placements(start, player_color, simulated_playfields);
            }

            // Resetting the in-place simulation on the other ring
            self.state[target_ring_index] = target_ring_backup;
        } else if let MoveDirection::OnRing { target_field_index } = direction {
            assert!(target_field_index < 8);

            // Set the empty neighbors value to the old one of the current index:
            self.state[start.ring_index] |= stone_color << (target_field_index * 2);

            if init_mill_count == 0 {
                simulated_playfields.push(*self);
            } else {
                self.add_simulated_placements(start, player_color, simulated_playfields);
            }

            /* if 0 < was_in_mill {
                let backup_after_first_move = self.state;

                'outer: for field_and_bitmask in empty_fields {
                    let _target_field_state = self.state[start.ring_index] & (3u16 << (target_field_index * 2));

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
            } */
        }

        // Resetting the in-place simulation
        self.state[start.ring_index] = start_ring_backup;
    }
}
