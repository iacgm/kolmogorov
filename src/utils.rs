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

	//returns the stack as a reversed vector
	pub fn rev_vec(&self) -> Vec<T>
	where
		T: Clone,
	{
		let mut vec = vec![];

		let mut node = &*self.0;
		while let Node::Cons(h, t) = node {
			vec.push(h.clone());
			node = &*t.0;
		}

		vec
	}
}

impl<T> From<Node<T>> for Stack<T> {
	fn from(value: Node<T>) -> Self {
		Self(value.into())
	}
}
