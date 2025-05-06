use std::ops::Range;

use rustc_hash::FxHashMap as HashMap;

#[derive(Default)]
pub struct OEISMap {
    pub seq: HashMap<usize, Vec<i32>>,
    pub kws: HashMap<usize, Vec<String>>,
}

pub struct OEISLoadOptions {
    pub required: Vec<&'static str>,
    pub disallow: Vec<&'static str>,
    pub max_val: i32,
    pub min_len: usize,
}

impl Default for OEISLoadOptions {
    fn default() -> Self {
        Self {
            required: vec!["nice", "easy", "core"],
            disallow: vec![
                "base", "bref", "cofr", "cons", "dumb", "fini", "full", "hard",
                "obsc", "word", "dupe",
            ],
            max_val: i32::MAX,
            min_len: 10,
        }
    }
}

pub fn load_oeis_def() -> std::io::Result<OEISMap> {
    load_oeis(&Default::default())
}

pub fn load_oeis(options: &OEISLoadOptions) -> std::io::Result<OEISMap> {
    const SEQS_FILE: &str = "data/stripped";
    const KEYS_FILE: &str = "data/keywords";

    let mut map = OEISMap::default();

    let keys_file = std::fs::read_to_string(KEYS_FILE)?;

    for line in keys_file.lines() {
        let name = &line[1..=6];

        let id = name.parse::<usize>().unwrap();

        let kws = line[8..].split(",").map(String::from).collect::<Vec<_>>();

        // .contains does not work for String/&str comparison
        let is_kw = |r| kws.iter().any(|s| s == r);

        if options.required.iter().all(is_kw)
            && !options.disallow.iter().any(is_kw)
        {
            map.kws.insert(id, kws);
        }
    }
    drop(keys_file);

    let seqs_file = std::fs::read_to_string(SEQS_FILE)?;
    for line in seqs_file.lines() {
        let mut words = line.trim().split(",");

        let name = words.next().unwrap().trim();

        let id = name[1..].parse::<usize>().unwrap();

        if !map.kws.contains_key(&id) {
            continue;
        }

        let mut nums = vec![];

        for word in words {
            if word.is_empty() {
                // Since each line ends in a comma
                continue;
            }

            let Ok(n) = word.parse::<i32>() else {
                break;
            };

            if n > options.max_val {
                break;
            }

            nums.push(n)
        }

        if nums.len() < options.min_len {
            continue;
        }

        map.seq.insert(id, nums);
    }

    Ok(map)
}
