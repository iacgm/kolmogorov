use kolmogorov::*;

mod languages;
use languages::*;

fn main() {
    let lang = NumLogic::new(2);

    let term = term!(f -> count(f)(k -> (prime(atom(k)))));

    let nps = [
        0, 1, 2, 2, 3, 3, 4, 4, 4, 4, 5, 5, 6, 6, 6, 6, 7, 7, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 10, 10,
        11, 11, 11, 11, 11, 11, 12, 12, 12, 12, 13, 13, 14, 14, 14, 14, 15, 15, 15, 15, 15, 15, 16,
        16, 16, 16, 16, 16, 17, 17, 18, 18, 18, 18, 18, 18, 19, 19, 19, 19, 20, 20, 21, 21, 21, 21,
        21, 21,
    ];

    for i in 2u32..70 {
        let term = term!([term] [:i]);
        let out = lang.context().evaluate(&term);
        let np = out.get::<u32>();

        assert_eq!(np, nps[i as usize - 1], "{} -> {}", i, np);
    }
}
