use kolmogorov::*;

mod languages;
use languages::*;

fn main() {
    let lang = NumLogic::new(2);

    let ty = ty!(Var => Bool);

    for n in 1.. {
        println!("Round {}", n);
        let start = std::time::Instant::now();

        let searcher = search(&lang, vec![], &ty, n);

        let mut count = 0;

        for (term, analysis) in searcher {
            count += 1;
            println!("\n{}", term);
            if let Analysis::Canonical(sem) = analysis {
                println!("≈ {}", sem);
            }
        }

        println!();

        println!(
            "These are all {:>6} known-distinct programs of type {} and size {}.",
            count, ty, n
        );

        let end = std::time::Instant::now();

        println!("Time elapsed: {}s", end.duration_since(start).as_secs_f32());

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
