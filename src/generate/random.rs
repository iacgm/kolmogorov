use super::*;

use rand::random_range;
use std::rc::Rc;

const SMALL_SIZE: usize = 15;

// Replaces a random small AST node with another AST of the same type
pub fn mutate(lang: Box<dyn Language>, term: &Term) -> Term {
	let ctxt = lang.context();

	let (replacement_node, metadata) = random_subnode(&ctxt, term, SMALL_SIZE);

	let new = uniform_sample(search(lang, &metadata.ty, metadata.size))
		.unwrap()
		.0;

	replace_subnode(term, replacement_node, new)
}

pub fn replace_subnode(dest: &Term, node_id: usize, src: Term) -> Term {
	fn helper(counter: &mut usize, dest: &Term, node_id: usize, src: Term) -> Term {
		*counter += 1;

		if *counter == node_id {
			return src;
		}

		use Term::*;
		match dest {
			Ref(r) => helper(counter, &(**r).borrow(), node_id, src),
			Lam(v, b) => Lam(v, helper(counter, b, node_id, src).into()),
			App(l, r) => {
				let l = &(**l).borrow().clone();
				let l = helper(counter, l, node_id, src.clone());
				let r = &(**r).borrow().clone();
				let r = helper(counter, r, node_id, src);
				App(l.into(), r.into())
			}
			_ => dest.clone(),
		}
	}

	helper(&mut 0, dest, node_id, src)
}

// Metadata for a subnode
struct Metadata {
	size: usize,
	ty: Rc<Type>,
	defs: VarsVec, // Variables in scope
}

// Reservoir sampling, again.
// We return the index of the subnode (using pre-order numbering) & its size
fn random_subnode(ctxt: &Context, term: &Term, max_size: usize) -> (usize, Metadata) {
	let mut selected_id: usize = 0;
	let mut metadata = None;
	let mut stack = vec![term.clone()];
	let mut counter = 1;

	while let Some(next) = stack.pop() {
		let size = next.size();

		if size <= max_size && random_range(0..counter) == 0 {
			selected_id = counter;
			metadata = Some(Metadata {
				size,
				ty: ctxt.infer_type(&next).into(),
				defs: vec![],
			});
		}

		use Term::*;
		match next {
			Ref(r) => stack.push(r.borrow().clone()),
			Lam(_, b) => stack.push((*b).clone()),
			App(l, r) => {
				stack.push(r.borrow().clone());
				stack.push(l.borrow().clone());
			}
			_ => (),
		}

		counter += 1;
	}

	(selected_id, metadata.unwrap())
}
