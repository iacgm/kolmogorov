pub mod iterative;
pub mod k_rec;
pub mod pure_iterative;
pub mod simple_map;

pub use iterative::*;
pub use k_rec::*;
pub use pure_iterative::*;
pub use simple_map::*;

use super::*;

use statrs::distribution::{Continuous, Normal};

pub struct SynthesisParameters {
    pub bias: SizeBias,
    pub score_factor: f64,
    pub iterations: usize,
}

pub struct MetropolisOutput<L: Language> {
    pub term: Term,
    pub iterations: usize,
    pub time: f64,
    pub num_correct: usize,
    pub score: Option<f64>,
    pub analysis: Analysis<L>,
}

impl Default for SynthesisParameters {
    fn default() -> Self {
        Self {
            bias: SizeBias::Unbiased,
            score_factor: 0.5,
            iterations: 50_000,
        }
    }
}

// Used to bias programs towards reasonable sizes / prevent runaway term sizes
#[derive(Clone, Copy)]
pub enum SizeBias {
    Unbiased,
    LinearBeyond { cutoff: usize, c: f64 },
    Norm { m: f64, s: f64 },
    DistAbs { mean: usize, c: f64 },
}

impl SizeBias {
    pub fn apply(self, score: f64, size: usize) -> f64 {
        use SizeBias::*;
        match self {
            Unbiased => score,
            LinearBeyond { cutoff, c } => {
                let punishment = -c * size.saturating_sub(cutoff) as f64;

                score * punishment.exp()
            }
            Norm { m, s } => {
                let normal = Normal::new(m, s).unwrap();

                score * normal.pdf(size as f64)
            }
            DistAbs { mean, c } => {
                let dist = if mean >= size {
                    mean - size
                } else {
                    size - mean
                };

                let punishment = -c * dist as f64;

                score * punishment.exp()
            }
        }
    }
}

impl<L: Language> MetropolisOutput<L> {
    pub fn display(&self) {
        let MetropolisOutput {
            term,
            iterations,
            time,
            num_correct,
            score,
            analysis,
        } = self;

        println!("Best Found: {}", &term);
        println!("Semantics:  {}", analysis);

        println!("Score: {:?} (or {:?} correct)", score, num_correct,);

        println!("Iterations: {}", iterations);
        println!("Time (s): {}", time);
        println!("Time (s/iter): {}", time / *iterations as f64);
    }
}
