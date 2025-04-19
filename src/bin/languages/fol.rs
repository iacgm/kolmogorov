use std::fmt::Display;
use std::rc::Rc;

use kolmogorov::*;

use super::polynomials::*;

type Number = Identifier;

#[derive(Debug, Clone)]
pub struct LogicLang {
    max_depth: usize,
    context: Context,
}

// An atomic formula
#[derive(Debug, Ord, PartialOrd, Clone, PartialEq, Eq, Hash)]
pub enum Predicate {
    Prime(Number),
    Divisor(Number, Number),
}

// A predicate, with a bool indicating whether it is negated
type Literal = (bool, Predicate);

type Conjunction = Vec<Literal>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Exists {
    limit: Number,
    body: Rc<LogicSems>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogicSems {
    Val(Number),
    And(Conjunction),                // A conjunction of Literals
    App(Identifier, Vec<LogicSems>), // Variables are not analyzed until they are contextualized
    Any(Exists),
    Abs(Identifier, Box<LogicSems>),
}

impl Language for LogicLang {
    // We track semantics AND the depth of each subterm
    type Semantics = LogicSems;

    const SMALL_SIZE: usize = 10;
    const LARGE_SIZE: usize = 18;

    fn context(&self) -> Context {
        self.context.clone()
    }

    fn svar(&self, v: Identifier, ty: &Type) -> Analysis<Self> {
        use Analysis::*;
        use Identifier::*;
        use LogicSems::*;
        use Type::*;
        match ty {
            // Disallow function variables
            Fun(_, _) => Malformed,
            Var(Name("N")) => Canonical(Val(v)),
            _ => Canonical(App(v, vec![])),
        }
    }

    fn sval(&self, _: &Value, _ty: &Type) -> Analysis<Self> {
        unimplemented!()
    }

    fn slam(
        &self,
        ident: Identifier,
        body: Analysis<Self>,
        ty: &Type,
    ) -> Analysis<Self> {
        use Analysis::*;
        use LogicSems::*;

        let Type::Fun(_, ret) = ty else {
            unreachable!()
        };

        if let Type::Fun(_, _) = &**ret {
            return Malformed;
        }

        Canonical(Abs(ident, Box::new(body.canon())))
    }

    fn sapp(
        &self,
        fun: Analysis<Self>,
        arg: Analysis<Self>,
        _ty: &Type,
    ) -> Analysis<Self> {
        use Analysis::*;
        use LogicSems::*;
        use Predicate::*;

        let fun = fun.canon();
        let arg = arg.canon();

        let sems = match fun {
            App(v, args) if v.as_str() == "prime" => {
                debug_assert!(args.is_empty());
                let Val(v) = arg else { unreachable!() };
                And(vec![(false, Prime(v))])
            }
            App(v, mut args) if v.as_str() == "divisor" && args.len() == 1 => {
                let (Val(p), Val(q)) = (args.remove(0), arg) else {
                    unreachable!()
                };

                And(vec![(false, Divisor(p, q))])
            }
            App(v, mut args) if v.as_str() == "and" && args.len() == 1 => {
                let (And(mut bools), And(rest)) = (args.remove(0), arg) else {
                    unreachable!()
                };

                bools.extend(rest);
                bools.sort();

                And(bools)
            }
            App(v, mut args) if v.as_str() == "exists" && args.len() == 1 => {
                let Val(limit) = args.remove(0) else {
                    unreachable!()
                };

                if arg.depth() == self.max_depth {
                    return Malformed;
                }

                let body = arg.into();
                Any(Exists { limit, body })
            }
            App(v, mut args) => {
                args.push(arg);
                App(v, args)
            }
            _ => unimplemented!(),
        };

        Canonical(sems)
    }
}

impl LogicLang {
    pub fn new(max_depth: usize) -> Self {
        let primitives = Self::all_functions();

        let context = Context::new(primitives);

        Self { max_depth, context }
    }

    pub fn all_functions() -> Vec<(Identifier, BuiltIn)> {
        let exists = builtin! {
            N => (N => Bool) => Bool
            ctxt |b, f| => {
                Term::val((0..b.get::<u32>()).any(|n| ctxt.evaluate(&term!([f] [:n])).get::<bool>()))
            }
        };

        let and = builtin! {
            Bool => Bool => Bool
            |a, b| => Term::val(a.get::<bool>() && b.get::<bool>())
        };

        let prime = builtin! {
            N => Bool
            |n| => Term::val(is_prime(n.get::<u32>()))
        };

        let divisor = builtin! {
            N => N => Bool
            |p, q| => {
                let p = p.get::<u32>();
                let q = q.get::<u32>();
                Term::val(p > 1 && q % p == 0)
            }
        };

        vec![
            ("exists".into(), exists),
            ("and".into(), and),
            ("prime".into(), prime),
            ("divisor".into(), divisor),
        ]
    }
}

impl LogicSems {
    pub fn depth(&self) -> usize {
        use LogicSems::*;
        match self {
            Any(Exists { body, .. }) => body.depth() + 1,
            App(_, args) => {
                args.iter().map(LogicSems::depth).max().unwrap_or(0)
            }
            _ => 0,
        }
    }
}

// Simple algorithm
fn is_prime(n: u32) -> bool {
    let mut i = 2;
    while i * i < n {
        if n % i == 0 {
            return false;
        }
        i += 1;
    }

    true
}

impl Display for LogicSems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
