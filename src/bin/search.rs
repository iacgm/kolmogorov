use kolmogorov::*;

mod languages;
use languages::*;

type Lang = NumLogic;

fn main() {
    let lang = Lang::new(2);
    let ty = ty!(Var => Bool);

    let mut cache = search::Cache::<Lang>::new();

    for n in 1.. {
        let start = std::time::Instant::now();

        let mut searcher = search::search_with_cache(&lang, vec![], &ty, n, cache);

        let count = searcher.by_ref().count();
        cache = searcher.cache();

        println!(
            "There are {:>6} known-distinct programs of type {} and size {}.",
            count, ty, n
        );

        let end = std::time::Instant::now();

        println!("Time elapsed: {}s", end.duration_since(start).as_secs_f32());
    }
}
