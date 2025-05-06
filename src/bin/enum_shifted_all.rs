use kolmogorov::*;

mod languages;
use languages::*;

mod utils;
use utils::oeis::*;

fn main() {
    let lang = NumLogic::new(2);
    let ty = ty!(Var => Num);

    let limit = 25;
    
    let oeis = load_oeis(&OEISLoadOptions {
        required: vec!["nonn"],
        min_len: limit,
        ..Default::default()
    })
    .unwrap();

    let conv = |v: Vec<i32>| v.into_iter().take(limit).map(|n| n as u32).collect::<Vec<u32>>();

    let mut seqs = oeis
        .seq
        .into_iter()
        .map(|(id, sq)| (id, conv(sq)))
        .collect::<Vec<_>>();
    seqs.sort_by_key(|(_id, sq)| sq.clone());

    let programs = (1..)
        .inspect(|n| println!("Searching size: {}", n))
        .flat_map(|n| search(&lang, vec![], &ty, n));

    for (program, analysis) in programs {
        let mut outs = vec![];
        for num in 1..=limit as u32 {
            let prog = term!([program] [:num]);

            let out = lang.context().evaluate(&prog).get::<u32>();

            outs.push(out);
        }

        let search = seqs.binary_search_by_key(&&outs, |(_id, sq)| sq);
        if search.is_ok() {
            println!("{} ≈ {}:", program, analysis);
        }
        while let Ok(i) = seqs.binary_search_by_key(&&outs, |(_id, sq)| sq) {
            let id = seqs.remove(i).0;
            println!("\tA{:06}", id);
        }
    }
}

