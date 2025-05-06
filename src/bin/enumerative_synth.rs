use kolmogorov::*;

mod languages;
use languages::*;

mod utils;
use utils::oeis::*;

fn main() {
    let lang = NumLogic::new(2);
    let ty = ty!(Var => Bool);

    let oeis = load_oeis(&OEISLoadOptions {
        required: vec!["nonn"],
        ..Default::default()
    })
    .unwrap();

    let key = 246655;

    let nums: Vec<u32> = oeis.seq[&key].iter().map(|n| *n as u32).collect();

    let limit = *nums.last().unwrap();

    let programs = (1..)
        .inspect(|n| println!("Searching size: {}", n))
        .flat_map(|n| search(&lang, vec![], &ty, n));

    let start = std::time::Instant::now();

    'next: for (program, analysis) in programs {
        for num in 0..limit {
            let prog = term!([program] [:num]);

            let out = lang.context().evaluate(&prog).get::<bool>();

            if out != nums.contains(&num) {
                continue 'next;
            }
        }
        println!("Found: {} ~= {}", program, analysis);
        break;
    }

    let end = std::time::Instant::now();

    println!("Total time: {}", end.duration_since(start).as_secs_f32());
}
