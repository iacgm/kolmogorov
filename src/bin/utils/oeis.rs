use rustc_hash::FxHashMap as HashMap;

#[derive(Default)]
pub struct OEISMap {
	seqs: HashMap<usize, Vec<i32>>,
}

impl OEISMap {
	pub fn load() -> std::io::Result<OEISMap> {
		const FILENAME: &str = "data/stripped";

		let mut map = OEISMap::default();

		let file = std::fs::read_to_string(FILENAME)?;

		'lines: for line in file.lines() {
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
					continue 'lines;
				};

				nums.push(n)
			}

			map.seqs.insert(id, nums);
		}

		Ok(map)
	}

	pub fn iter(&self) -> impl Iterator<Item = (&usize, &Vec<i32>)> {
		self.seqs.iter()
	}
}
