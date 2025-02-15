use rand::Rng;

pub fn with_probability(p: f64) -> bool {
	random() < p
}

pub fn random() -> f64 {
	rand::random::<f64>()
}

// Select one random element from iterator (using reservoir sampling)
// Also returns # of elements. Needed for Metropolis-Hastings
pub fn reservoir_sample<T>(mut iter: impl Iterator<Item = T>) -> (usize, Option<T>) {
	let mut res = iter.next();
	let mut count = 0;
	let mut rng = rand::thread_rng();

	for item in iter {
		count += 1;
		let r = rng.gen_range(0..=count);

		if r == 0 {
			res = Some(item);
		}
	}

	(count, res)
}
