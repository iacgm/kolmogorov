use super::*;

use rand::random_range;
use rustc_hash::FxHashMap as HashMap;

// Probability of replacing a variable with another
const REPLACE_VAR: f64 = 0.7;
// Probability of replacing a small (non-variable) subterm with another of equal size
const REPLACE_SMALL: f64 = 0.29;
// Probability of replacing a larger subterm with anohter, potentially of different size
// This is much more computationally expensive and can erase a lot of progress, but also
// allows us to exit local minima (we must calculate g(x'|x) & g(x|x'), involving a census
// of terms we don't even use, so it should be much more unlikely than the others).
#[allow(unused)]
const REPLACE_LARGE: f64 = 1. - REPLACE_VAR - REPLACE_SMALL;

// Max size of `small` terms. (TODO: Make language-dependent)
const SMALL_SIZE: usize = 15;

// Max size of `large` terms. (TODO: Make language-dependent)
const LARGE_SIZE: usize = 20;

// How often we print out progress
const PRINT_SPACE: usize = 100;

pub fn metropolis<F: FnMut(&Term) -> f64, L: Language>(
	lang: &L,
	start: &Term,
	ty: &Type,
	mut scorer: F,
	iterations: usize,
) -> Term {
	let mut candidate = start.clone();
	let mut score = scorer(start);

	let mut best_candidate = start.clone();
	let mut best_score = 0.;

	for i in 0..iterations {
		if i % PRINT_SPACE == 0 {
			println!("Metropolis progress: {}/{}", i, iterations);
		}

		// g_ratio = g(x|x') / g(x'|x)
		let Some((proposal, g_ratio)) = mutate(lang, &candidate, ty) else {
			continue;
		};

		let proposal_score = scorer(&proposal);

		if proposal_score > best_score {
			best_score = proposal_score;
			best_candidate = proposal.clone();
		}

		let score_ratio = proposal_score / score;

		let acceptance_prob = score_ratio * g_ratio;

		if with_probability(acceptance_prob) {
			candidate = proposal;
			score = proposal_score;
		}
	}

	best_candidate
}

// Mutates a &Term. Also returns g(x|x') / g(x'|x) [where x' is the proposal]
pub fn mutate<L: Language>(lang: &L, term: &Term, ty: &Type) -> Option<(Term, f64)> {
	let ctxt = lang.context();

	use Replacement::*;
	match Replacement::choose_replacement_kind() {
		HVar => {
			let (replacement_node, annotation, _) = random_subnode(&ctxt, term, ty, 1, 1);

			let (_, proposal) = reservoir_sample(search(
				lang,
				annotation.defs,
				&annotation.ty,
				annotation.size,
			));

			let (new_term, _analysis) = proposal.unwrap();

			let candidate = replace_subnode(term, replacement_node, new_term);

			Some((candidate, 1.))
		}
		Small => {
			let (replacement_node, annotation, _) = random_subnode(&ctxt, term, ty, 2, SMALL_SIZE);

			let (_, replacement) = reservoir_sample(search(
				lang,
				annotation.defs,
				&annotation.ty,
				annotation.size,
			));

			let (new_term, _analysis) = replacement.unwrap();

			let candidate = replace_subnode(term, replacement_node, new_term);

			Some((candidate, 1.))
		}
		Large => {
			let (replacement_node, annotation, subnode_count) =
				random_subnode(&ctxt, term, ty, 2, LARGE_SIZE);

			let replacement_size = rand::random_range(1..LARGE_SIZE);

			println!(
				"Counting (Used):   {} {} {:?}",
				annotation.ty, replacement_size, annotation.defs
			);

			let (new_count, replacement) = reservoir_sample(search(
				lang,
				annotation.defs.clone(),
				&annotation.ty,
				replacement_size,
			));

			let (replacement, _analysis) = replacement?;

			let proposal = replace_subnode(term, replacement_node, replacement);

			println!(
				"Counting (Unused): {} {} {:?}",
				annotation.ty, annotation.size, annotation.defs
			);

			let old_count = search(lang, annotation.defs, &annotation.ty, annotation.size).count();

			// g1 = g(x' | x)
			let g1 = g(subnode_count, new_count);

			let (_, _, subnode_count) = random_subnode(&ctxt, term, ty, 2, LARGE_SIZE);
			//g2 = g(x | x')
			let g2 = g(subnode_count, old_count);

			Some((proposal, g2 / g1))
		}
	}
}

// g(x2 | x1)
fn g(x1_subnode_count: usize, x2_num_replacement_terms: usize) -> f64 {
	let prob_subnode_selected = 1. / x1_subnode_count as f64;

	let prob_replacement_generated = 1. / x2_num_replacement_terms as f64;

	prob_subnode_selected * prob_replacement_generated
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
// Returns (node_id, annotation, small_node_count)
fn random_subnode(
	ctxt: &Context,
	term: &Term,
	ty: &Type,
	min_size: usize,
	max_size: usize,
) -> (usize, Annotation, usize) {
	let mut selected_id: usize = 0;
	let mut stack = vec![(term.clone(), term as *const Term)];
	let mut counter = 1;

	let metadata = annotate_term(term, ctxt, ty);

	let ptr = term as *const Term;
	let mut annotation = metadata.get(&ptr).unwrap();

	while let Some((next, ptr)) = stack.pop() {
		let size = next.size();

		if (min_size..=max_size).contains(&size) && random_range(0..counter) == 0 {
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

	(selected_id, annotation.clone(), counter - 1)
}

#[derive(Clone, Debug)]
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

enum Replacement {
	HVar,
	Small,
	Large,
}

impl Replacement {
	pub fn choose_replacement_kind() -> Self {
		let rand = random();

		if rand < REPLACE_VAR {
			Self::HVar
		} else if rand < REPLACE_VAR + REPLACE_SMALL {
			Self::Small
		} else {
			Self::Large
		}
	}
}
