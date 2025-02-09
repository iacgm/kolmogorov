use super::*;

use rand::random_range;
use rustc_hash::FxHashMap as HashMap;

const SMALL_SIZE: usize = 15;

// Replaces a random small AST node with another AST of the same type
pub fn mutate(lang: Box<dyn Language>, term: &Term, ty: &Type) -> Term {
	let ctxt = lang.context();

	let (replacement_node, annotation) = random_subnode(&ctxt, term, ty, SMALL_SIZE);

	let new = uniform_sample(search(
		lang,
		annotation.defs,
		&annotation.ty,
		annotation.size,
	))
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

// Reservoir sampling, again.
// We return the index of the subnode (using pre-order numbering) & its size
fn random_subnode(ctxt: &Context, term: &Term, ty: &Type, max_size: usize) -> (usize, Annotation) {
	let mut selected_id: usize = 0;
	let mut stack = vec![(term.clone(), term as *const Term)];
	let mut counter = 1;

	let metadata = annotate_term(term, ctxt, ty);

	let ptr = term as *const Term;
	let mut annotation = metadata.get(&ptr).unwrap();

	while let Some((next, ptr)) = stack.pop() {
		let size = next.size();

		if size <= max_size && random_range(0..counter) == 0 {
			selected_id = counter;
			annotation = metadata.get(&ptr).unwrap();
		}

		use Term::*;
		match next {
			Ref(r) => stack.push((r.borrow().clone(), r.as_ptr())),
			Lam(_, b) => stack.push(((*b).clone(), b.as_ref() as *const Term)),
			App(l, r) => {
				stack.push((r.borrow().clone(), r.as_ptr()));
				stack.push((l.borrow().clone(), l.as_ptr()));
			}
			_ => (),
		}

		counter += 1;
	}

	(selected_id, annotation.clone())
}

#[derive(Clone)]
struct Annotation {
	size: usize,
	ty: Type,
	defs: VarsVec, // Variables in scope
}

type Metadata = HashMap<*const Term, Annotation>;

// Can fail if Term is not in beta-nf
fn annotate_term(term: &Term, ctxt: &Context, ty: &Type) -> Metadata {
	fn annotate(
		term: &Term,
		ctxt: &Context,
		ty: Option<&Type>,
		map: &mut Metadata,
		decls: &VarsVec,
	) {
		let ptr = term as *const Term;

		if map.contains_key(&ptr) {
			return;
		}

		use Term::*;
		let annotation = match term {
			Ref(r) => {
				annotate(&r.borrow(), ctxt, ty, map, decls);

				let ptr = r.as_ptr() as *const Term;

				map.get(&ptr).unwrap().clone()
			}
			Num(_) => Annotation {
				size: 1,
				defs: decls.clone(),
				ty: Type::Int,
			},
			Var(v) => {
				if let Some((_, v_ty)) = decls.iter().find(|(s, _)| v == s) {
					Annotation {
						size: 1,
						ty: (**v_ty).clone(),
						defs: decls.clone(),
					}
				} else if let Some(builtin) = ctxt.get(v) {
					Annotation {
						size: 1,
						ty: (*builtin.ty).clone(),
						defs: decls.clone(),
					}
				} else {
					panic!("Undeclared variable")
				}
			}
			Lam(v, b) => {
				let ty = ty.unwrap().clone();

				let Type::Fun(arg, ret) = ty.clone() else {
					unimplemented!()
				};

				let mut decls = decls.clone();
				decls.push((v, arg.clone()));

				annotate(b, ctxt, Some(ret.as_ref()), map, &decls);

				Annotation {
					size: term.size(),
					ty,
					defs: decls,
				}
			}
			App(l, r) => {
				let f = l.as_ptr() as *const Term;

				annotate(&l.borrow(), ctxt, None, map, decls);

				let f_note = map.get(&f).unwrap().clone();

				let Type::Fun(arg, ret) = f_note.ty else {
					unreachable!()
				};

				annotate(&r.borrow(), ctxt, Some(&*arg), map, decls);

				Annotation {
					size: term.size(),
					ty: (*ret).clone(),
					defs: f_note.defs,
				}
			}
		};

		map.insert(ptr, annotation);
	}

	let mut map = Metadata::default();
	annotate(term, ctxt, Some(ty), &mut map, &vec![]);
	map
}
