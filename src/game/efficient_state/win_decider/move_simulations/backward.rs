use crate::game::{
    efficient_state::{DirectionToCheck, EfficientPlayField, MoveDirection},
    PlayerColor,
};

impl EfficientPlayField {
    pub fn simulate_backward_move_get_playfields(
        &mut self,
        fields_to_place: &Vec<(usize, u16)>,
        start_ring_index: usize,
        start_fields_index: u16,
        direction: MoveDirection,
        player_color: PlayerColor,
    ) -> Vec<EfficientPlayField> {
        let mut simulated_playfields = Vec::<EfficientPlayField>::new();
        let stone_color: u16 = player_color.into();

        // To rollback the in-situ changes on self
        let start_ring_backup = self.state[start_ring_index];

        // Check for mills before the move has taken place
        let was_in_mill = self.get_mill_count(
            start_ring_index,
            start_fields_index / 2, //hier
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
}
