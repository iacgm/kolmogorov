use kolmogorov::*;

mod languages;
use languages::*;

mod utils;
use utils::oeis::*;

fn main() {
    let lang = NumLogic::new(2);
    let ty = ty!(Var => Num);

    let oeis = load_oeis(&OEISLoadOptions {
        required: vec!["nonn", "nice", "easy"],
        ..Default::default()
    })
    .unwrap();

    let key = 10;

    let nums: Vec<u32> = oeis.seq[&key].iter().map(|n| *n as u32).collect();

    dbg!(&nums);

    let limit = nums.len() as u32;

    let programs = (1..)
        .inspect(|n| println!("Searching size: {}", n))
        .flat_map(|n| search(&lang, vec![], &ty, n));

    let start = std::time::Instant::now();

    'next: for (program, analysis) in programs {
        for i in 1..limit {
            let prog = term!([program] [:i]);

            let out_prog = lang.context().evaluate(&prog);

            let out = out_prog.get::<u32>();

            if out != nums[i as usize - 1] {
                continue 'next;
            }
        }
        println!("Found: {} ~= {}", program, analysis);
        break;
    }

    let end = std::time::Instant::now();

    println!("Total time: {}", end.duration_since(start).as_secs_f32());
}
