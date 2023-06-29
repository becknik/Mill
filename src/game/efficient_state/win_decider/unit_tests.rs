use crate::game::efficient_state::EfficientPlayField;

#[test]
fn number_of_base_won_playfields_is_correct_test() {
    let mut incremental_won_set = EfficientPlayField::generate_white_won_configurations(9);

    assert_eq!(7825361, incremental_won_set.len());

    EfficientPlayField::add_white_won_configurations_enclosed(9, &mut incremental_won_set);

    assert_eq!(567794, incremental_won_set.len() - 7825361);
    assert_eq!(8393155, incremental_won_set.len());
    // TODO Shouldn't we get less because we filter out some of the unreachable fields?
}

#[test]
fn t3vs3_all_won_loose_playfields_count_correct() {
    let (won, lost) = EfficientPlayField::generate_won_configs_black_and_white(3);

    assert_eq!(140621, won.len());
    assert_eq!(28736, lost.len());
}
