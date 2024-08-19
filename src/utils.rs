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
			Cons(t, h) => write!(f, "{:?}:{}", t, h),
		}
	}
}

#[derive(Clone)]
enum Node<T> {
	Nil,
	Cons(Stack<T>, T),
}

impl<T> Stack<T> {
	pub fn one(v: T) -> Self {
		Node::Cons(Node::Nil.into(), v).into()
	}

	pub fn cons(&self, v: T) -> Self {
		Node::Cons(Stack(self.0.clone()), v).into()
	}

	pub fn is_nil(&self) -> bool {
		use Node::*;
		match *self.0 {
			Nil => true,
			Cons(_, _) => false,
		}
	}

	pub fn to_vec(&self) -> Vec<T>
	where
		T: Clone,
	{
		use Node::*;
		match &*self.0 {
			Nil => vec![],
			Cons(t, h) => {
				let mut v = t.to_vec();
				v.push(h.clone());
				v
			}
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
	pub fn build_term(self) -> Term {
		use Node::*;
		match Rc::unwrap_or_clone(self.0) {
			Nil => unimplemented!(),
			Cons(t, h) if t.is_nil() => h,
			Cons(t, h) => {
				let mut v = t.to_vec();
				v.push(h);
				v.reverse();
				Term::App(v)
			}
		}
	}
}
