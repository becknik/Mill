use fnv::FnvHashSet;

use crate::game::{efficient_state::EfficientPlayField, PlayerColor};


impl EfficientPlayField {
    fn generate_won_configs_white(max_stone_count: usize) -> FnvHashSet<EfficientPlayField> {
        let mut won_set = FnvHashSet::<EfficientPlayField>::default();
        Self::generate_3_canon_mills(&mut won_set);

        for canon_mill in won_set {
            for i in 0..24 {
                let ring_index = (i / 8) as usize;
                let field_index = i % 8;

                // to avoid placing stones onto already present mills
                if canon_mill.get_field_state_at(ring_index, field_index, false) != 0 {
                    continue;
                }

                let mut config = canon_mill.clone();
                //adding first black stone
                config.state[ring_index] |= 2u16 << (field_index * 2);

                for j in (i + 1)..24 {
                    let ring_index = (j / 8) as usize;
                    let field_index = j % 8;

                	if canon_mill.get_field_state_at(ring_index, field_index, false) != 0 {
                        continue;
                    }

                    let mut config = config.clone();
                    //adding second black stone
                    config.state[ring_index] |= 2u16 << (field_index * 2);

                    won_set.insert(config.get_canonical_form());

                    // white stones must be placed before black ones => start_index = 0
                    config.distribute_stones_and_add(PlayerColor::White, max_stone_count - 3, 0, &mut won_set);
                }
            }
        }

        Self::add_white_won_configurations_enclosed(max_stone_count, &mut won_set);

        won_set
    }

    /// Hard-coded generation of the only 3 unique mill playfield configuration
    /// Uses the the mirroring & rotation of the play field and swapping of ring index which is done by the canonical form generation
    fn generate_3_canon_mills(set: &mut FnvHashSet<EfficientPlayField>) {
        let mut pf = EfficientPlayField::default();
        pf.set_field_state(2, 7, 1);
        pf.set_field_state(2, 0, 1);
        pf.set_field_state(2, 1, 1);
        set.insert(pf);

        let mut pf = EfficientPlayField::default();
        pf.set_field_state(1, 7, 1);
        pf.set_field_state(1, 0, 1);
        pf.set_field_state(1, 1, 1);
        set.insert(pf);

        let mut pf = EfficientPlayField::default();
        pf.set_field_state(2, 0, 1);
        pf.set_field_state(1, 0, 1);
        pf.set_field_state(0, 0, 1);
        set.insert(pf);
    }

	// TODO Might be wrong due to removing the immutable part?
	/// Places the amount of stones ion the playfield, starting on `start_index` from left to the right
	///
	///  - `amount_of_stones` is the recursion depth of this function
    fn distribute_stones_and_add(
        &self,
        stone_color: PlayerColor,
        amount_of_stones: usize,
        start_index: u16,
        set: &mut FnvHashSet<EfficientPlayField>,
    ) {
        if 0 < amount_of_stones {
            for i in start_index..24 {
                let ring_index = (i / 8) as usize;
                let field_index = i % 8;

                if self.get_field_state_at(ring_index, field_index, false) != 0 {
                    continue;
                }

                let ring_backup = self.state[ring_index];
                self.state[ring_index] |= <PlayerColor as Into<u16>>::into(stone_color) << (field_index * 2);

                set.insert(self.get_canonical_form());

                if 24 <= start_index {
                    return;
                }
				// Recursive call with one stones less to the next start_index
				else if 1 < amount_of_stones {
                    self.distribute_stones_and_add(stone_color, amount_of_stones - 1, i + 1, set);
                }
                self.state[ring_index] = ring_backup;
            }
        }
    }
}