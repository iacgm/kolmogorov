use std::rc::Rc;

#[derive(Clone)]
pub struct Stack<T>(Rc<Node<T>>);

pub enum Node<T> {
	Nil,
	Cons(T, Rc<Self>),
}

impl<T> Stack<T> {
	pub fn from(v: T) -> Self {
		Node::Cons(v, Node::Nil.into()).into()
	}

	pub fn cons(&self, v: T) -> Self {
		Node::Cons(v, self.0.clone()).into()
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
			node = t;
		}

		vec
	}
}

impl<T> From<Node<T>> for Stack<T> {
	fn from(value: Node<T>) -> Self {
		Self(value.into())
	}
}
