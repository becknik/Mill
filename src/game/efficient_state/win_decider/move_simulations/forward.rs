//! To containerize all possible in-place functions with a backup to be applied after the operations executed.
//! We call them "simulations"

use crate::game::efficient_state::{DirectionToCheck, EfficientPlayField, FieldPos, MoveDirection};

use smallvec::SmallVec;
use DirectionToCheck::*;
use MoveDirection::*;

use super::TO_TAKE_VEC_SIZE;

impl EfficientPlayField {
    /// Simulates the move implicitly specified in the function header by it's parameters, but also all implications
    /// like the takes of moves which end up in a closed mill.
    ///
    /// TODO assert the start FieldPos field_index in abstract representation (<8)
    pub fn simulate_possible_forward_moves_for(
        &mut self,
        fields_to_take: &SmallVec<[FieldPos; TO_TAKE_VEC_SIZE]>,
        start: FieldPos,
        direction: MoveDirection,
        color: u16,
        simulated_playfields: &mut Vec<EfficientPlayField>,
    ) {
        // TODO assert!(start.field_index < 8);
        let start = FieldPos {
            field_index: start.field_index / 2,
            ..start
        };

        let start_ring_backup = self.state[start.ring_index];

        // Clear out the start index
        self.state[start.ring_index] &= !(3u16 << (start.field_index * 2));

        if let AcrossRings { target_ring_index } = direction {
            let target_ring_backup = self.state[target_ring_index];

            // Moving a stone onto the field on the other ring
            self.state[target_ring_index] |= color << (start.field_index * 2);

            let target_pos = FieldPos {
                ring_index: target_ring_index,
                ..start
            };
            self.add_simulated_moves(target_pos, direction, color, simulated_playfields, fields_to_take);

            self.state[target_ring_index] = target_ring_backup;
        } else if let MoveDirection::OnRing { target_field_index } = direction {
            let target_field_index = target_field_index / 2;
            assert!(target_field_index < 8);

            // Set the empty neighbors value to the old one of the current index:
            self.state[start.ring_index] |= color << (target_field_index * 2);

            let target_pos = FieldPos {
                field_index: target_field_index,
                ..start
            };
            self.add_simulated_moves(target_pos, direction, color, simulated_playfields, fields_to_take);
        }
        // End simulation by applying backup state
        self.state[start.ring_index] = start_ring_backup;
    }

    /// Check for mills on the simulated move which is done in-place in self.
    ///
    /// If there is a new mill due to the simulated move, take the free-to-take positions stones contained in the `fields_to_take`
    /// vector by calling the `add_simulated_takes` function
    ///
    /// TODO Silly name
    fn add_simulated_moves(
        &mut self,
        start: FieldPos,
        direction: MoveDirection,
        color: u16,
        simulated_playfields: &mut Vec<EfficientPlayField>,
        fields_to_take: &SmallVec<[FieldPos; TO_TAKE_VEC_SIZE]>,
    ) {
        let possible_mills_count = match direction {
            MoveDirection::OnRing { target_field_index } => self.get_mill_count(
                start.ring_index,
                target_field_index,
                OnAndAcrossRings { player_color: color },
            ),
            AcrossRings { target_ring_index } => self.get_mill_count(target_ring_index, start.field_index, OnRing),
        };

        if 0 < possible_mills_count {
            self.add_simulated_takes(simulated_playfields, fields_to_take);
        } else {
            simulated_playfields.push(self.clone());
        }
    }

    /// Simulates the moves taking on element from the `fields_to_take` set & adds them to the simulated_playfields vector
    fn add_simulated_takes(
        &mut self,
        simulated_playfields: &mut Vec<EfficientPlayField>,
        fields_to_take: &SmallVec<[FieldPos; TO_TAKE_VEC_SIZE]>,
    ) {
        let state_backup = self.state;

        for field in fields_to_take {
            self.state[field.ring_index] &= !(3u16 << (field.field_index * 2));

            simulated_playfields.push(self.clone());
            self.state = state_backup;
        }
    }
}
