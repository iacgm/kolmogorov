use kolmogorov::*;

mod languages;
use languages::*;

mod utils;
use utils::*;

fn main() {
    let lang = NumLogic::new(2);

    let term = term!(n -> exists n (a -> exists n (b -> and (prime b) (and (prime (atom a)) (eq (mul (atom a) (atom b)) n)))));

    dbg!(term.size());

    let oeis = load_oeis(&OEISLoadOptions {
        required: vec!["nonn"],
        ..Default::default()
    })
    .unwrap();

    let key = dbg!(1358);

    let nps: Vec<u32> = oeis.seq[&key].iter().map(|n| *n as u32).collect();

    let limit = *nps.last().unwrap();

    for i in 0u32..=limit {
        let term = term!([term] [:i]);
        let out = lang.context().evaluate(&term);
        let np = out.get::<bool>();

        assert_eq!(np, nps.contains(&i), "{} -> {}", i, np);
    }
}
