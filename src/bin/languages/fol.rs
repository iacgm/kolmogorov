#![allow(dead_code)]

use std::fmt::{write, Display};
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
    var: Identifier,
    bound: Number,
    body: Rc<LogicSems>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogicSems {
    Val(Number),
    And(Conjunction),                // A conjunction of Literals
    App(Identifier, Vec<LogicSems>), // Variables are not analyzed until they are contextualized
    Any(Exists),
    Abs(Identifier, Rc<LogicSems>),
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
            Fun(_, _) if self.context.get(v).is_none() => Malformed,
            Var(Name("N")) => Canonical(Val(v)),
            _ => Canonical(App(v, vec![])),
        }
    }

    fn sval(&self, _: &Value, _ty: &Type) -> Analysis<Self> {
        unimplemented!()
    }

    fn slam(&self, ident: Identifier, body: Analysis<Self>, ty: &Type) -> Analysis<Self> {
        use Analysis::*;
        use LogicSems::*;

        let Type::Fun(_, ret) = ty else {
            unreachable!()
        };

        if let Type::Fun(_, _) = &**ret {
            return Malformed;
        }

        Canonical(Abs(ident, Rc::new(body.canon())))
    }

    fn sapp(&self, fun: Analysis<Self>, arg: Analysis<Self>, _ty: &Type) -> Analysis<Self> {
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
                    return Malformed;
                };

                bools.extend(rest);
                bools.sort();
                bools.dedup();

                And(bools)
            }
            App(v, mut args) if v.as_str() == "exists" && args.len() == 1 => {
                let Val(limit) = args.remove(0) else {
                    unreachable!()
                };

                let Abs(var, body) = arg else {
                    return Malformed;
                };

                if body.depth() == self.max_depth {
                    return Malformed;
                }

                Any(Exists {
                    var,
                    bound: limit,
                    body,
                })
            }
            App(v, mut args) => {
                args.push(arg);
                App(v, args)
            }
            Abs(_, _) => return Malformed,
            _ => unimplemented!("{} @ {}", fun, arg),
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
                Term::val((1..b.get::<u32>()).any(|n| ctxt.evaluate(&term!([f] [:n])).get::<bool>()))
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
            App(_, args) => args.iter().map(LogicSems::depth).max().unwrap_or(0),
            _ => 0,
        }
    }
}

// Simple algorithm
fn is_prime(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    let mut i = 2;
    while i * i < n {
        if n % i == 0 {
            return false;
        }
        i += 1;
    }

    true
}

impl Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Predicate::*;
        match self {
            Prime(identifier) => write!(f, "Prime({})", identifier),
            Divisor(p, q) => write!(f, "{}|{}", p, q),
        }
    }
}

impl Display for LogicSems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LogicSems::*;
        match self {
            Val(identifier) => write!(f, "{}", identifier),
            And(items) => {
                write!(f, "{}", items[0].1)?;
                for cond in &items[1..] {
                    write!(f, "∧{}", cond.1)?;
                }
                Ok(())
            }
            App(identifier, items) => {
                write!(f, "{}", identifier)?;
                for item in items {
                    match item {
                        Abs(_, _) => write!(f, "{}", item)?,
                        _ => write!(f, "({})", item)?,
                    }
                }
                Ok(())
            }
            Any(exists) => {
                write!(f, "∃{}<{} [{}]", exists.var, exists.bound, exists.body)
            }
            Abs(identifier, body) => {
                write!(f, "({} ", identifier)?;
                let mut next = body;
                while let Abs(v, body) = &**next {
                    write!(f, "{} ", v)?;
                    next = body;
                }
                write!(f, "-> {})", next)
            }
        }
    }
}
