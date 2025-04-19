use super::*;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

pub type Thunk = Rc<RefCell<Term>>;

pub type Value = Rc<dyn TermValue>;

#[derive(Clone, Debug)]
pub enum Term {
    Val(Value),
    Var(Identifier),
    Lam(Identifier, Rc<Term>),
    App(Thunk, Thunk),

    // Ref:
    // Transparent indirection to another term (May be deleted in the process of other operations)
    // Occasionally useful for a faithful implementation of graph reduction.
    // May eventually be removed during optimization.
    Ref(Thunk),
}

impl Term {
    pub fn val<T: TermValue>(x: T) -> Self {
        Self::Val(Rc::new(x))
    }

    pub fn deep_clone(&self) -> Self {
        use Term::*;
        match self {
            Ref(r) => (**r).borrow().deep_clone(),
            Val(n) => Val(n.clone()),
            Var(v) => Var(*v),
            Lam(v, b) => Lam(*v, b.clone()),
            App(l, r) => App(
                (**l).borrow().deep_clone().into(),
                (**r).borrow().deep_clone().into(),
            ),
        }
    }

    pub fn instantiate_var(&self, var: Identifier, thunk: &Thunk) -> Term {
        use Term::*;
        match self {
            Val(n) => Val(n.clone()),
            Lam(v, b) => {
                if *v == var {
                    Lam(*v, b.clone())
                } else {
                    Lam(*v, b.instantiate_var(var, thunk).into())
                }
            }
            Var(v) => {
                if *v == var {
                    Ref(thunk.clone())
                } else {
                    Var(*v)
                }
            }
            Ref(next) => {
                let next = &*(**next).borrow();
                next.instantiate_var(var, thunk)
            }
            App(l, r) => App(
                (**l).borrow().instantiate_var(var, thunk).into(),
                (**r).borrow().instantiate_var(var, thunk).into(),
            ),
        }
    }

    pub fn size(&self) -> usize {
        use Term::*;
        match self {
            Ref(r) => (**r).borrow().size(),
            Val(_) | Var(_) => 1,
            Lam(_, b) => 1 + b.size(),
            App(l, r) => 1 + l.borrow().size() + r.borrow().size(),
        }
    }

    pub fn leaf_val(&self) -> Option<Value> {
        use Term::*;
        match self {
            Ref(r) => r.borrow().leaf_val(),
            Val(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn get<T: TermValue + Clone>(&self) -> T {
        let rc = self.leaf_val().unwrap();
        let any = rc.as_any();
        any.downcast_ref::<T>().unwrap().clone()
    }

    pub fn try_get<T: TermValue + Clone>(&self) -> Option<T> {
        let rc = self.leaf_val()?;
        let any = rc.as_any();
        let out = any.downcast_ref::<T>()?;
        Some(out.clone())
    }

    pub fn in_beta_normal_form(&self) -> bool {
        use Term::*;
        match self {
            Ref(r) => r.borrow().in_beta_normal_form(),
            Val(_) | Var(_) => true,
            Lam(_, b) => b.in_beta_normal_form(),
            App(l, r) => {
                !l.borrow().is_lam()
                    && l.borrow().in_beta_normal_form()
                    && r.borrow().in_beta_normal_form()
            }
        }
    }

    fn is_lam(&self) -> bool {
        use Term::*;
        match self {
            Ref(r) => r.borrow().is_lam(),
            Lam(_, _) => true,
            _ => false,
        }
    }
}

pub fn cast<T: TermValue>(rc: &Value) -> Option<&T> {
    rc.as_any().downcast_ref()
}

pub trait TermValue: Any + Debug + Display {
    // For some reason requiring PartialEq is bad but this is okay?
    // If you don't love Rust at its trait bound restrictions,
    // you don't deserve it at its... uhhh... nevermind...
    fn is_eq(&self, other: &Value) -> bool;
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any + Debug + Display + PartialEq> TermValue for T {
    fn is_eq(&self, other: &Value) -> bool {
        let Some(other) = other.as_any().downcast_ref::<T>() else {
            return false;
        };
        self == other
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

//Syntactic equality, not α-equality (might be useful to implement eventually)
impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        use Term::*;
        match (self, other) {
            (Ref(r), t) | (t, Ref(r)) => &*(**r).borrow() == t,
            (Val(a), Val(b)) => a.is_eq(b),
            (Var(a), Var(b)) => a == b,
            (Lam(va, ba), Lam(vb, bb)) => va == vb && ba == bb,
            (App(ll, lr), App(rl, rr)) => {
                let ll = &ll.borrow();
                let lr = &lr.borrow();
                let rl = &rl.borrow();
                let rr = &rr.borrow();
                **ll == **rl && **lr == **rr
            }
            _ => false,
        }
    }
}

impl From<Term> for Thunk {
    fn from(value: Term) -> Self {
        Rc::new(value.into())
    }
}

use std::fmt::*;
impl Display for Term {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        use Term::*;
        match self {
            Ref(r) => write!(fmt, "{}", (**r).borrow()),
            Val(k) => write!(fmt, "{}", k),
            Var(v) => write!(fmt, "{}", v),
            Lam(v, b) => {
                write!(fmt, "(\\{}", v)?;
                let mut r = &**b;
                while let Lam(v, next) = r {
                    write!(fmt, " {}", v)?;
                    r = &**next;
                }
                write!(fmt, " -> {}", r)?;
                write!(fmt, ")")
            }
            App(l, r) => {
                write!(fmt, "{}({})", (**l).borrow(), (**r).borrow())
            }
        }
    }
}
