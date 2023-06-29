use criterion::{criterion_group, criterion_main, Criterion};

use muehle::game::efficient_state::EfficientPlayField;
use nanorand::{Rng, WyRand};

fn make_playfield_random(pf: &mut EfficientPlayField) {
    let mut rng = WyRand::default();

    for i in 0..3 {
        for j in 0..7 {
            // w-1:b-1:e-2 is more ralistical
            let randome_number = match rng.generate_range(1..=4) {
                1 => 1,
                2 => 2,
                0 | 3 | 4 | 5 => continue,
                _ => panic!(),
            };

            pf.set_field_state(i, j, randome_number);
        }
    }
}

fn canonical_form_benchmark(c: &mut Criterion) {
    let mut test_play_fields = [EfficientPlayField::default(); 2048];
    test_play_fields.iter_mut().for_each(|pf| make_playfield_random(pf));

    c.bench_function("canonical_form1", move |b| {
        b.iter(|| {
            test_play_fields.iter_mut().for_each(|pf| {
                pf.get_canon_form();
            })
        });
    });
}

fn move_triple_benchmark(c: &mut Criterion) {
    let mut test_play_fields = [EfficientPlayField::default(); 2048];
    test_play_fields.iter_mut().for_each(|pf| make_playfield_random(pf));

    c.bench_function("move_triple1", move |b| {
        b.iter(|| {
            test_play_fields.iter_mut().for_each(|pf| {
                pf.get_move_triple(muehle::game::PlayerColor::White);
            })
        });
    });
}

//criterion_group!(benches, canonical_form_benchmark);
criterion_group!(benches, move_triple_benchmark);
criterion_main!(benches);
