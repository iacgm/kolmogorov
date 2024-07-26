use std::{
	fmt::{Debug, Display},
	rc::Rc,
};

#[derive(Clone)]
pub struct Stack<T>(Rc<Node<T>>);

impl<T: Display> Debug for Stack<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use Node::*;
		match &*self.0 {
			Nil => Ok(()),
			Cons(h, t) => write!(f, "{:?}:{}", t, h),
		}
	}
}

pub enum Node<T> {
	Nil,
	Cons(T, Stack<T>),
}

impl<T> Stack<T> {
	pub fn one(v: T) -> Self {
		Node::Cons(v, Node::Nil.into()).into()
	}

	pub fn cons(&self, v: T) -> Self {
		Node::Cons(v, Stack(self.0.clone())).into()
	}

	pub fn is_nil(&self) -> bool {
		use Node::*;
		match *self.0 {
			Nil => true,
			Cons(_, _) => false,
		}
	}
}

impl<T> From<Node<T>> for Stack<T> {
	fn from(value: Node<T>) -> Self {
		Self(value.into())
	}
}

use super::lambda::Term;
impl Stack<Term> {
	//returns the stack as a reversed vector
	pub fn build_term(&self) -> Term {
		use Node::*;
		match &*self.0 {
			Nil => unimplemented!(),
			Cons(h, t) if t.is_nil() => h.clone(),
			Cons(h, t) => t.build_term().applied_to(h.clone()),
		}
	}
}
