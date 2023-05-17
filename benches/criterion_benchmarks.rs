
use criterion::{Criterion, criterion_group, criterion_main};

use muehle::game::efficient_state::EfficientPlayField;
use nanorand::{WyRand, Rng};

fn make_playfield_random(pf: &mut EfficientPlayField) {
	let mut rng = WyRand::default();

	for i in 0..3 {
		for j in 0..7 {
			let randome_number = rng.generate_range(0..=2);
			if randome_number == 0 {continue;}
			pf.set_field(i, j, randome_number);
		}
	}
}

fn get_canonical_form_benchmark1(c: &mut Criterion) {
	let mut test_play_fields = [EfficientPlayField::default(); 2000];
	test_play_fields.iter_mut().for_each(|pf| make_playfield_random(pf));

	c.bench_function(
		"get_canonical_form_standard", move |b| {
			b.iter(|| {
				test_play_fields.iter_mut().for_each(|pf| {pf.get_canonical_form();})
			});
		}
	);
}

criterion_group!(benches, get_canonical_form_benchmark1);
criterion_main!(benches);