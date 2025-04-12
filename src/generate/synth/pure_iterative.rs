use super::*;
pub fn pure_iterative<L, O>(
    lang: L,
    seed: O,
    examples: impl Iterator<Item = O>,
    start: Term,
    ty: Type,
    settings: SynthesisParameters,
    options: Options,
) -> MetropolisOutput
where
    L: Language,
    O: TermValue + Clone,
{
    let seed_term = Term::val(seed);

    let examples = examples.map(std::rc::Rc::new).collect::<Vec<_>>();

    let num_examples = examples.len();

    let lang_ctxt = lang.context();

    let int_scorer = |t: &Term| {
        let mut num_correct = 0;

        let mut yielded = seed_term.clone();

        for o in examples.iter() {
            let program = term! {
                [t] [yielded]
            };

            let evaled = lang_ctxt.evaluate(&program);

            let Some(output) = evaled.leaf_val() else {
                unimplemented!("Term `{}` did not evaluate to value.", evaled);
            };

            if o.is_eq(&output) {
                num_correct += 1;
            }

            yielded = Term::Val(o.clone());
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
    let (iterations, term) =
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
    }
}
