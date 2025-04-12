use super::*;

use std::fmt::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Analysis<L: Language> {
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

    fn sval(&self, _: &Rc<dyn TermValue>) -> Analysis<Self> {
        Analysis::Unique
    }

    fn svar(&self, _: Identifier) -> Analysis<Self> {
        Analysis::Unique
    }

    fn slam(
        &self,
        _ident: Identifier,
        _body: Analysis<Self>,
    ) -> Analysis<Self> {
        Analysis::Unique
    }

    fn sapp(
        &self,
        _fun: Analysis<Self>,
        _arg: Analysis<Self>,
    ) -> Analysis<Self> {
        Analysis::Unique
    }

    fn analyze(&self, term: &Term) -> Analysis<Self> {
        use Term::*;
        match term {
            Val(v) => self.sval(v),
            Var(v) => self.svar(*v),
            Lam(i, b) => self.slam(*i, self.analyze(b)),
            App(l, r) => {
                self.sapp(self.analyze(&l.borrow()), self.analyze(&r.borrow()))
            }
            Ref(r) => self.analyze(&r.borrow()),
        }
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
