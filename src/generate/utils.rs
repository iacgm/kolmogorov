// Select one random element from iterator (using reservoir sampling)
pub fn uniform_sample<T>(iter: impl Iterator<Item = T>) -> Option<T> {
	let mut res = None;

	for (i, item) in iter.enumerate() {
		let r = rand::random_range(0..=i);

		if r == 0 {
			res = Some(item);
		}
	}

	res
}
