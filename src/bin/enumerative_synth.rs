use kolmogorov::*;

mod languages;
use languages::*;

fn main() {
    let lang = NumLogic::new(2);
    let ty = ty!(Var => Bool);

    let nums: Vec<u32> = vec![
        1, 2, 3, 4, 5, 7, 8, 9, 11, 13, 16, 17, 19, 23, 25, 27, 29, 31, 32, 37,
        41, 43, 47, 49, 53, 59, 61, 64, 67, 71, 73, 79, 81, 83, 89, 97, 101,
        103, 107, 109, 113, 121, 125, 127, 128, 131, 137, 139, 149, 151, 157,
        163, 167, 169, 173, 179, 181, 191, 193, 197, 199, 211, 223, 227,
    ];

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
