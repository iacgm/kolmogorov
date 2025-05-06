use std::fmt::{write, Display};
use std::process::id;
use std::rc::Rc;

use kolmogorov::*;

use super::polynomials::*;

#[derive(Debug, Clone)]
pub struct NumLogic {
    max_depth: usize,
    context: Context,
}

type Var = Identifier;

#[derive(PartialOrd, PartialEq, Eq, Ord, Clone, Hash, Debug)]
pub enum Atom {
    Var(Var),
    Pow(Var, Var),
}

type Atoms = Vec<Atom>;

// An atomic formula
#[derive(Debug, Ord, PartialOrd, Clone, PartialEq, Eq, Hash)]
pub enum Predicate {
    Prime(Atoms),
    Divisor(Atoms, Atoms),
    Eq(Atoms, Atoms),
    Less(Atoms, Atoms),
}

// A predicate, with a bool indicating whether it is negated
type Literal = (bool, Predicate);

type Conjunction = Vec<Literal>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Reducer {
    Existential,
    Sigma,
    Count,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Reduction {
    reducer: Reducer,
    var: Identifier,
    bound: Var,
    body: Rc<NumLogicSems>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NumLogicSems {
    Mul(Atoms),
    And(Conjunction),                   // A conjunction of Literals
    App(Identifier, Vec<NumLogicSems>), // Variables are not analyzed until they are contextualized
    Red(Reduction),
    Abs(Identifier, Rc<NumLogicSems>),
}

impl Language for NumLogic {
    // We track semantics AND the depth of each subterm
    type Semantics = NumLogicSems;

    const SMALL_SIZE: usize = 15;
    const LARGE_SIZE: usize = 30;

    fn context(&self) -> Context {
        self.context.clone()
    }

    fn svar(&self, v: Identifier, ty: &Type) -> Analysis<Self> {
        use Analysis::*;
        use Atom::*;
        use Identifier::*;
        use NumLogicSems::*;

        match ty {
            // Disallow function variables
            Type::Fun(_, _) if self.context.get(v).is_none() => Malformed,
            Type::Var(Name("Atom" | "Var")) => Canonical(Mul(vec![Var(v)])),
            _ => Canonical(App(v, vec![])),
        }
    }

    fn sval(&self, _: &Value, _ty: &Type) -> Analysis<Self> {
        unimplemented!()
    }

    fn slam(&self, ident: Identifier, body: Analysis<Self>, ty: &Type) -> Analysis<Self> {
        use Analysis::*;
        use NumLogicSems::*;

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
        use Atom::*;
        use NumLogicSems::*;
        use Predicate::*;

        let fun = fun.canon();
        let arg = arg.canon();

        let sems = match fun {
            App(v, args) if v.as_str() == "atom" => {
                debug_assert!(args.is_empty());
                arg
            }
            App(v, args) if v.as_str() == "prime" => {
                debug_assert!(args.is_empty());

                let Mul(s) = arg else { unreachable!() };
                And(vec![(false, Prime(s))])
            }
            App(v, mut args) if v.as_str() == "pow" && args.len() == 1 => {
                let (Mul(mut l), Mul(mut r)) = (args.remove(0), arg) else {
                    unreachable!()
                };

                let (Var(b), Var(k)) = (l.remove(0), r.remove(0)) else {
                    unreachable!()
                };

                Mul(vec![Pow(b, k)])
            }
            App(v, mut args) if v.as_str() == "mul" && args.len() == 1 => {
                let (Mul(mut l), Mul(r)) = (args.remove(0), arg) else {
                    unreachable!()
                };
                l.extend(r);
                l.sort();

                Mul(l)
            }
            App(v, mut args) if v.as_str() == "less" && args.len() == 1 => {
                let (Mul(p), Mul(q)) = (args.remove(0), arg) else {
                    unreachable!()
                };

                And(vec![(false, Less(p, q))])
            }
            App(v, mut args) if v.as_str() == "eq" && args.len() == 1 => {
                let (Mul(p), Mul(q)) = (args.remove(0), arg) else {
                    unreachable!()
                };

                let eq = if p <= q { Eq(p, q) } else { Eq(q, p) };

                And(vec![(false, eq)])
            }
            App(v, mut args) if v.as_str() == "divisor" && args.len() == 1 => {
                let (Mul(p), Mul(q)) = (args.remove(0), arg) else {
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
            App(v, mut args)
                if ["sigma", "count", "exists"].contains(&v.as_str()) && args.len() == 1 =>
            {
                let Mul(mut sum) = args.remove(0) else {
                    unreachable!()
                };
                let Var(limit) = sum.remove(0) else {
                    unreachable!()
                };

                let Abs(var, body) = arg else {
                    return Malformed;
                };

                if body.depth() == self.max_depth {
                    return Malformed;
                }

                use Reducer::*;
                let reducer = match v.as_str() {
                    "exists" => Existential,
                    "sigma" => Sigma,
                    "count" => Count,
                    _ => unreachable!(),
                };

                Red(Reduction {
                    reducer,
                    var,
                    bound: limit,
                    body,
                })
            }
            App(v, _) if ["num", "atom", "to_bool"].contains(&v.as_str()) => arg,
            App(v, mut args) => {
                args.push(arg);
                App(v, args)
            }
            Abs(_, _) => return Malformed,
            _ => unimplemented!("{:?} @ {:?}", fun, arg),
        };

        Canonical(sems)
    }
}

#[allow(dead_code)]
impl NumLogic {
    pub fn new(max_depth: usize) -> Self {
        let primitives = Self::all_functions();

        let context = Context::new(primitives);

        Self { max_depth, context }
    }

    pub fn all_functions() -> Vec<(Identifier, BuiltIn)> {
        let int = |t: &Term| t.get::<u32>();
        let bln = |t: &Term| t.get::<bool>();

        let atom = builtin! {
            Var => Atom
            |v| => v.clone()
        };

        let pow = builtin! {
            Var => Var => Atom
            |c, p| => Term::val(int(&c).checked_pow(int(&p)).unwrap_or(0))
        };

        let mul = builtin! {
            Atom => Atom => Atom
            |l, r| => Term::val(int(&l).checked_mul(int(&r)).unwrap_or(0))
        };

        let exists = builtin! {
            Var => (Var => Bool) => Bool
            ctxt |b, f| => {
                Term::val((1..=int(&b)).any(|n| bln(&ctxt.evaluate(&term!([f] [:n])))))
            }
        };

        let num = builtin! {
            Atom => Num
            |n| => n.clone()
        };

        let sigma = builtin! {
            Var => (Var => Num) => Num
            ctxt |b, f| => {
                Term::val((1..=int(&b)).map(|n| int(&ctxt.evaluate(&term!([f] [:n])))).sum::<u32>())
            }
        };

        let count = builtin! {
            Var => (Var => Bool) => Num
            ctxt |b, f| => {
                Term::val((1..=int(&b)).filter(|&n| bln(&ctxt.evaluate(&term!([f] [:n])))).count() as u32)
            }
        };

        let to_bool = builtin! {
            Pred => Bool
            |p| => p.clone()
        };

        let and = builtin! {
            Pred => Bool => Bool
            |a, b| => Term::val(bln(&a) && bln(&b))
        };

        let prime = builtin! {
            Atom => Pred
            |n| => Term::val(is_prime(int(&n)))
        };

        let divisor = builtin! {
            Atom => Atom => Pred
            |p, q| => {
                let p = int(&p);
                let q = int(&q);
                Term::val(p > 1 && q % p == 0)
            }
        };

        let eq = builtin! {
            Atom => Atom => Pred
            |l, r| => Term::val(int(&l) == int(&r))
        };

        let less = builtin! {
            Atom => Atom => Pred
            |l, r| => Term::val(int(&l) < int(&r))
        };

        vec![
            ("num".into(), num),
            ("atom".into(), atom),
            ("bool".into(), to_bool),
            ("pow".into(), pow),
            ("mul".into(), mul),
            ("count".into(), count),
            ("sigma".into(), sigma),
            ("eq".into(), eq),
            ("less".into(), less),
            ("exists".into(), exists),
            ("and".into(), and),
            ("prime".into(), prime),
            ("divisor".into(), divisor),
        ]
    }
}

impl NumLogicSems {
    pub fn depth(&self) -> usize {
        use NumLogicSems::*;
        match self {
            Red(Reduction { body, .. }) => body.depth() + 1,
            App(_, args) => args.iter().map(NumLogicSems::depth).max().unwrap_or(0),
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
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 1;
    }

    true
}

impl Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Atom::Var(identifier) => write!(f, "{}", identifier),
            Atom::Pow(b, k) => write!(f, "{}^{}", b, k),
        }
    }
}

fn fmt_mul(f: &mut std::fmt::Formatter<'_>, sum: &Atoms) -> std::fmt::Result {
    write!(f, "({}", sum[0])?;
    for s in &sum[1..] {
        write!(f, "*{}", s)?;
    }
    write!(f, ")")
}

impl Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Predicate::*;
        match self {
            Prime(sum) => {
                write!(f, "Prime")?;
                fmt_mul(f, sum)
            }
            Divisor(p, q) => {
                fmt_mul(f, p)?;
                write!(f, "|")?;
                fmt_mul(f, q)
            }
            Eq(l, r) => {
                fmt_mul(f, l)?;
                write!(f, "=")?;
                fmt_mul(f, r)
            }
            Less(l, r) => {
                fmt_mul(f, l)?;
                write!(f, "<")?;
                fmt_mul(f, r)
            }
        }
    }
}

impl Display for NumLogicSems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use NumLogicSems::*;
        match self {
            Mul(sum) => fmt_mul(f, sum),
            And(items) => {
                write!(f, "{}", items[0].1)?;
                for cond in &items[1..] {
                    write!(f, " && {}", cond.1)?;
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
            Abs(identifier, body) => {
                write!(f, "({} ", identifier)?;
                let mut next = body;
                while let Abs(v, body) = &**next {
                    write!(f, "{} ", v)?;
                    next = body;
                }
                write!(f, "-> {})", next)
            }
            Red(reduction) => {
                use Reducer::*;
                write!(
                    f,
                    "{}{}<{} [{}]",
                    match reduction.reducer {
                        Sigma => "Σ",
                        Count => "#",
                        Existential => "∃",
                    },
                    reduction.var,
                    reduction.bound,
                    reduction.body
                )
            }
        }
    }
}
