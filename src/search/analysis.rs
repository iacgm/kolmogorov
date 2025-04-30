use super::*;

use std::fmt::*;

#[derive(Clone, Debug)]
pub enum Analysis<L: Language>
where
    L::Semantics: Semantics,
{
    Malformed, // Reject Term entirely (i.e, unnecessarily complex)
    Unique,    // Allow, but did not construct canonical form
    Canonical(L::Semantics), // Group into equivalence class by canonical form
}

pub trait Language: Sized + Clone + Debug {
    type Semantics: Semantics + Sized;

    // Max size of `small` terms. (TODO: Make language-dependent)
    const SMALL_SIZE: usize = 5;

    // Max size of `large` terms. (TODO: Make language-dependent)
    const LARGE_SIZE: usize = 10;

    fn context(&self) -> Context;

    fn sval(&self, _: &Value, _ty: &Type) -> Analysis<Self> {
        Analysis::Unique
    }

    fn svar(&self, _: Identifier, _ty: &Type) -> Analysis<Self> {
        Analysis::Unique
    }

    fn slam(
        &self,
        _ident: Identifier,
        _body: Analysis<Self>,
        _ty: &Type,
    ) -> Analysis<Self> {
        Analysis::Unique
    }

    fn sapp(
        &self,
        _fun: Analysis<Self>,
        _arg: Analysis<Self>,
        _ty: &Type,
    ) -> Analysis<Self> {
        Analysis::Unique
    }
}

impl<L: Language> Analysis<L> {
    pub fn canon(self) -> L::Semantics {
        use Analysis::*;
        match self {
            Canonical(c) => c,
            _ => panic!("`canon` called on {:?}", self),
        }
    }

    pub fn malformed(&self) -> bool {
        matches!(self, Self::Malformed)
    }
}

impl<L> Display for Analysis<L>
where
    L: Language,
    L::Semantics: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use Analysis::*;
        match self {
            Unique => write!(f, "Unique"),
            Malformed => write!(f, "Malformed"),
            Canonical(sem) => write!(f, "Canonical({})", sem),
        }
    }
}

impl<L: Language> PartialEq for Analysis<L> {
    fn eq(&self, other: &Self) -> bool {
        use Analysis::*;
        match (self, other) {
            (Unique, Unique) | (Malformed, Malformed) => true,
            (Canonical(l), Canonical(r)) => l == r,
            _ => false,
        }
    }
}
