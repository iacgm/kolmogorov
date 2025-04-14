use std::fmt::Display;

use kolmogorov::*;

use super::polynomials::*;

type Polynomial = Sum;

#[derive(Debug, Clone)]
pub struct LogicLang {
    max_depth: usize,
    functions: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Predicate {
    Prime(Polynomial),
    Eq(Polynomial, Polynomial),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogicSems {
    Poly(Polynomial),
    Conj(Vec<(bool, Predicate)>),
    Appl(Identifier, Vec<LogicSems>), //Variables are not analyzed until they are contextualized
}

impl Language for LogicLang {
    type Semantics = LogicSems;

    const SMALL_SIZE: usize = 10;
    const LARGE_SIZE: usize = 18;

    fn context(&self) -> Context {
        let funcs = Self::all_functions();
        Context::new(
            funcs
                .into_iter()
                .filter(|(n, _)| self.functions.contains(&n)),
        )
    }
}

impl LogicLang {
    pub fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            functions: Self::all_functions().into_iter().map(|t| t.0).collect(),
        }
    }

    pub fn all_functions() -> Vec<(Identifier, BuiltIn)> {
        let exists = builtin! {
            Bound => (N => Bool) => Bool
            |b| => Term::val((0..b.get::<u32>()).any(|n| true))
        };

        vec![("∃<=".into(), exists)]
    }
}

impl Display for LogicSems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
