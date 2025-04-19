use std::collections::VecDeque;

use super::*;
pub fn k_rec<L, O>(
    lang: L,
    k: usize,
    examples: impl Iterator<Item = O>,
    start: Term,
    ty: Type,
    settings: SynthesisParameters,
    options: Options,
) -> MetropolisOutput<L>
where
    L: Language,
    O: TermValue + Clone,
{
    let mut seeds = examples.map(Term::val::<O>).collect::<Vec<_>>();

    let examples = seeds.split_off(k);

    let num_examples = examples.len();

    let lang_ctxt = lang.context();

    let int_scorer = |t: &Term| {
        let mut num_correct = 0;

        let mut prevs = VecDeque::from(seeds.clone());

        for o in examples.iter() {
            let mut program = t.clone();
            for prev in prevs.iter() {
                program = Term::App(program.into(), prev.clone().into());
            }

            let evaled = lang_ctxt.evaluate(&program);

            let Some(output) = evaled.leaf_val() else {
                unimplemented!("term `{}` did not evaluate to value.", evaled);
            };

            if o.get::<O>().is_eq(&output) {
                num_correct += 1;
            }

            prevs.pop_front();
            prevs.push_back(o.clone());
        }

        num_correct
    };

    let scorer = |term: &Term| {
        let num_correct = int_scorer(term);

        if num_examples == num_correct {
            return None;
        }

        let prob_score = (settings.score_factor * num_correct as f64).exp();
        Some(settings.bias.apply(prob_score, term.size()))
    };

    let start_time = std::time::Instant::now();
    let (iterations, term, analysis) =
        metropolis(&lang, &start, &ty, scorer, settings.iterations, options);
    let end_time = std::time::Instant::now();

    let num_correct = int_scorer(&term);
    let score = scorer(&term);

    MetropolisOutput {
        term,
        iterations,
        time: end_time.duration_since(start_time).as_secs_f64(),
        num_correct,
        score,
        analysis,
    }
}
