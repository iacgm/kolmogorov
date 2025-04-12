use rustc_hash::FxHashMap as HashMap;

const MIN_EXAMPLE_COUNT: usize = 10;

pub type OEISMap = HashMap<usize, Vec<i32>>;

pub fn load_oeis() -> std::io::Result<OEISMap> {
	const FILENAME: &str = "data/stripped";

	let mut map = OEISMap::default();

	let file = std::fs::read_to_string(FILENAME)?;

	for line in file.lines() {
		let mut words = line.trim().split(",");

		let name = words.next().unwrap().trim();
		let id = name[1..].parse::<usize>().unwrap();

		let mut nums = vec![];

		for word in words {
			if word.is_empty() {
				// Since each line ends in a comma
				continue;
			}

			let Ok(n) = word.parse::<i32>() else {
				break;
			};

			nums.push(n)
		}

		if nums.len() < MIN_EXAMPLE_COUNT {
			continue;
		}

		map.insert(id, nums);
	}

	Ok(map)
}
