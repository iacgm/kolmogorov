use super::*;
use std::rc::Rc;

type BuiltInFunc = Rc<dyn Fn(&Context, &[Thunk]) -> Option<Term>>;

#[derive(Clone)]
pub struct BuiltIn {
    pub n_args: usize,
    pub func: BuiltInFunc,
    pub ty: Rc<Type>,
}
