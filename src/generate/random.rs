use super::*;

use rustc_hash::FxHashMap as HashMap;
use statrs::distribution::Discrete;
use std::rc::Rc;

// Probability of replacing a variable with another
const REPLACE_VAR: f64 = 0.5;
// Probability of replacing a small (non-variable) subterm with another of equal size
const REPLACE_SMALL: f64 = 0.40;
// Probability of replacing a larger subterm with anohter, potentially of different size
// This is much more computationally expensive and can erase a lot of progress, but also
// allows us to exit local minima (we must calculate g(x'|x) & g(x|x'), involving a census
// of terms we don't even use, so it should be much more unlikely than the others).
#[allow(unused)]
const REPLACE_LARGE: f64 = 1. - REPLACE_VAR - REPLACE_SMALL;

// How often we print out progress
const PRINT_FREQ: usize = 100;

// If F returns None, we stop immediately
pub fn metropolis<F: FnMut(&Term) -> Option<f64>, L: Language>(
	lang: &L,
	start: &Term,
	ty: &Type,
	mut scorer: F,
	iterations: usize,
) -> (usize, Term) {
	let ctxt = lang.context();

	let mut i = 0;

	let mut candidate = ImmutableTerm::from(start);
	let mut candidate_metadata = annotate(&ctxt, &start.into(), ty, &vec![]);

	let Some(mut score) = scorer(start) else {
		return (i, start.clone());
	};

	let mut best_candidate = start.clone();
	let mut best_score = 0.;

	let mut cache = SizeCache::default();

	while i <= iterations {
		i += 1;

		if i % PRINT_FREQ == 0 {
			println!(
				"Metropolis progress: {}/{}. Size {}",
				i,
				iterations,
				candidate.size()
			);
		}

		let mut proposal = candidate.clone();
		let mut metadata = candidate_metadata.clone();

		// g_ratio = g(x|x') / g(x'|x)
		let Some(g_ratio) = mutate(lang, &ctxt, &mut proposal, &mut metadata, &mut cache) else {
			continue;
		};

		let proposal_term = proposal.term();
		let Some(proposal_score) = scorer(&proposal_term) else {
			return (i, proposal_term);
		};

		if proposal_score > best_score {
			best_score = proposal_score;
			best_candidate = proposal_term;
		}

		let score_ratio = proposal_score / score;

		let acceptance_prob = dbg!(score_ratio * g_ratio);

		if dbg!(with_probability(acceptance_prob)) {
			candidate = proposal;
			candidate_metadata = metadata;
			score = proposal_score;
		}
	}

	(i, best_candidate)
}

// Mutates a &Term. Also returns g(x|x') / g(x'|x) [where x' is the proposal]
fn mutate<L: Language>(
	lang: &L,
	ctxt: &Context,
	term: &mut ImmutableTerm,
	meta: &mut Metadata,
	cache: &mut SizeCache<L>,
) -> Option<f64> {
	println!();
	println!("orig={}", term);
	use MutationTy::*;
	match dbg!(MutationTy::choose_replacement_kind()) {
		k @ HVar | k @ Small => {
			let (lo, hi) = if k == HVar {
				(1, 1)
			} else {
				(2, L::SMALL_SIZE)
			};

			let (replacement_id, replacement_node, note, _) = random_subnode(term, meta, lo, hi)?;

			let (_, sample) = cache.sample(lang, note.decls.clone(), &note.ty, note.size);

			let (replacement, _analysis) = sample?;

			let replacement_meta = annotate(ctxt, &replacement, &note.ty, &note.decls);

			*replacement_node = replacement;

			repair_metadata(term, meta, replacement_id, replacement_meta);

			println!("term is {}", term);

			Some(1.)
		}
		Large => {
			use rand::distributions::Distribution;
			use statrs::distribution::Binomial;

			let lo = 2;
			let hi = L::LARGE_SIZE;

			let (replacement_id, replacement_node, note, subnode_count) =
				random_subnode(term, meta, lo, hi)?;

			let ratio = note.size as f64 / L::LARGE_SIZE as f64;

			let size_distr = Binomial::new(ratio, L::LARGE_SIZE as u64).ok()?;
			let replacement_size: u64 = size_distr.sample(&mut rand::thread_rng());
			let replacement_size = replacement_size as usize;

			dbg!(replacement_size);

			let (new_count, sample) =
				cache.sample(lang, note.decls.clone(), &note.ty, replacement_size);

			let (replacement, _analysis) = sample?;

			let replacement_meta = annotate(ctxt, &replacement, &note.ty, &note.decls);
			*replacement_node = replacement;
			
			repair_metadata(term, meta, replacement_id, replacement_meta);

			let old_count = cache.query_count(lang, note.decls, &note.ty, note.size);

			// g1 = g(x' | x)
			let g1 = g::<L>(subnode_count, replacement_size, note.size, new_count);

			let subnode_count = meta.iter().filter(|s| (lo..=hi).contains(&s.size)).count();

			//g2 = g(x | x')
			let g2 = g::<L>(subnode_count, note.size, replacement_size, old_count);

			println!("term is {}", term);

			Some(g2 / g1)
		}
	}
}

// g(x2 | x1)
fn g<L: Language>(
	x1_subnode_count: usize,
	delta_size: usize,
	replaced_size: usize,
	x2_num_replacement_terms: usize,
) -> f64 {
	use statrs::distribution::Binomial;

	let prob_subnode_selected = 1. / x1_subnode_count as f64;

	let ratio = replaced_size as f64 / L::LARGE_SIZE as f64;

	let size_distr = Binomial::new(ratio, L::LARGE_SIZE as u64).unwrap();

	let prob_size_selected = size_distr.pmf(delta_size as u64);

	let prob_replacement_generated = 1. / x2_num_replacement_terms as f64;

	prob_subnode_selected * prob_size_selected * prob_replacement_generated
}

fn repair_metadata(
	term: &ImmutableTerm,
	meta: &mut Metadata,
	replacement_id: usize,
	replacement_metadata: Metadata,
) {
	fn traverse(term: &ImmutableTerm, id: usize, decls: &mut VarsVec, meta: &mut Metadata) {
		use ImmutableTerm::*;
		match term {
			ILam(v, b) => {
				let Type::Fun(arg, _) = &*meta[id].ty else {
					unreachable!()
				};

				decls.push((*v, arg.clone()));
				traverse(b, id + 1, decls, meta);
				decls.pop();

				meta[id].size = 1 + meta[id + 1].size;
				meta[id].decls = decls.clone();
			}
			IApp(l, r) => {
				let l_id = id + 1;
				traverse(l, l_id, decls, meta);
				let l_size = meta[l_id].size;

				let r_id = l_id + l_size;
				traverse(r, r_id, decls, meta);
				let r_size = meta[r_id].size;

				meta[id].size = 1 + l_size + r_size;
				meta[id].decls = decls.clone();
			}
			_ => (),
		}
	}

	println!("term={}", term);

	println!("repl_meta = {:?}", replacement_metadata);
	
	let old_size = meta[replacement_id].size;
	
	meta.splice(
		replacement_id..replacement_id + old_size,
		replacement_metadata,
	);

	println!("midswap =  {:?}", meta);

	traverse(term, 0, &mut vec![], meta);
	println!("meta={:?}", meta);
}

// Returns (id, ref, note, small_count)
pub fn random_subnode<'a>(
	term: &'a mut ImmutableTerm,
	metadata: &Metadata,
	min_size: usize,
	max_size: usize,
) -> Option<(usize, &'a mut ImmutableTerm, Annotation, usize)> {
	fn get_id<'a>(
		term: &'a mut ImmutableTerm,
		id: usize,
		metadata: &Metadata,
	) -> &'a mut ImmutableTerm {
		fn traverse<'a>(
			term: &'a mut ImmutableTerm,
			id: usize,
			metadata: &Metadata,
			term_id: usize,
		) -> &'a mut ImmutableTerm {
			if term_id == id {
				return term;
			}

			use ImmutableTerm::*;
			match term {
				ILam(_, b) => traverse(Rc::make_mut(b), id, metadata, term_id + 1),
				IApp(l, r) => {
					let l_id = term_id + 1;
					let l_size = metadata[l_id].size;

					let r_id = l_id + l_size;

					if id < r_id {
						traverse(Rc::make_mut(l), id, metadata, l_id)
					} else {
						traverse(Rc::make_mut(r), id, metadata, r_id)
					}
				}
				_ => unreachable!(),
			}
		}

		traverse(term, id, metadata, 0)
	}

	let range = min_size..=max_size;

	let (small_count, selection) = reservoir_sample(
		metadata
			.iter()
			.enumerate()
			.filter(|(_, s)| range.contains(&s.size))
			.map(|(i, s)| (i, s.clone())),
	);

	let (choice_id, note) = selection.unwrap();

	println!("id:   {}", choice_id);
	println!("note: {:?}", note);
	println!("meta: {:?}", metadata);

	let choice = get_id(term, choice_id, metadata);

	Some((choice_id, choice, note, small_count))
}

// Annotations in pre-order.
type Metadata = Vec<Annotation>;

// Fails if term is not in beta-nf
pub fn annotate(ctxt: &Context, term: &ImmutableTerm, ty: &Type, decls: &VarsVec) -> Metadata {
	fn traverse(
		ctxt: &Context,
		term: &ImmutableTerm,
		ty: Option<&Type>,
		metadata: &mut Metadata,
		decls: &mut VarsVec,
	) {
		use ImmutableTerm::*;

		let id = metadata.len();
		metadata.push(Annotation::default());

		let annotation = match term {
			INum(_) => Annotation {
				size: 1,
				ty: Type::Int.into(),
				decls: decls.clone(),
			},
			IVar(v) => {
				let v_ty = if let Some((_, v_ty)) = decls.iter().find(|(s, _)| v == s) {
					v_ty.clone()
				} else if let Some(builtin) = ctxt.get(*v) {
					builtin.ty.clone()
				} else {
					panic!("Undeclared variable: {}", v)
				};

				if let Some(ty) = ty {
					debug_assert!(&*v_ty == ty);
				}

				Annotation {
					size: 1,
					ty: v_ty,
					decls: decls.clone(),
				}
			}
			ILam(v, b) => {
				let ty = ty.unwrap().clone();

				let Type::Fun(arg, ret) = ty.clone() else {
					unimplemented!()
				};

				decls.push((*v, arg.clone()));

				traverse(ctxt, b, Some(ret.as_ref()), metadata, decls);

				decls.pop();

				let ty = Rc::from(ty);
				Annotation {
					size: term.size(),
					ty,
					decls: decls.clone(),
				}
			}
			IApp(l, r) => {
				let l_id = metadata.len();

				traverse(ctxt, l, None, metadata, decls);

				// Since we use pre-ordered annotations as data.
				let f_note = metadata[l_id].clone();

				let Type::Fun(arg, ret) = &*f_note.ty else {
					unreachable!()
				};

				traverse(ctxt, r, Some(arg), metadata, decls);

				Annotation {
					size: term.size(),
					ty: ret.clone(),
					decls: f_note.decls.clone(),
				}
			}
		};

		metadata[id] = annotation;
	}

	assert!(term.in_beta_normal_form());

	let mut metadata = Vec::with_capacity(term.size());

	traverse(ctxt, term, Some(ty), &mut metadata, &mut decls.clone());

	metadata
}

#[derive(Clone)]
pub struct Annotation {
	size: usize,
	ty: Rc<Type>,
	decls: VarsVec, // Variables in scope
}

impl Default for Annotation {
	fn default() -> Self {
		Self {
			size: 0,
			ty: Type::Int.into(),
			decls: vec![],
		}
	}
}

use std::fmt::*;
impl Debug for Annotation {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		write!(f, "{{|{}|, {}, {:?} }}", self.size, self.ty, self.decls)
	}
}

#[derive(Debug, PartialEq, Eq)]
enum MutationTy {
	HVar,
	Small,
	Large,
}

impl MutationTy {
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

type CtxtCache<L> = HashMap<(Type, usize), CacheEntry<L>>;

struct SizeCache<L: Language> {
	map: HashMap<VarsVec, CtxtCache<L>>,
}

enum CacheEntry<L: Language> {
	Explicit(Vec<(ImmutableTerm, Analysis<L>)>),
	Count(usize),
}

impl<L: Language> SizeCache<L> {
	const MAX_IN_MEM: usize = 32;

	pub fn sample(
		&mut self,
		lang: &L,
		mut decls: VarsVec,
		ty: &Type,
		size: usize,
	) -> (usize, Option<(ImmutableTerm, Analysis<L>)>) {
		use CacheEntry::*;

		decls.sort();
		let query = (ty.clone(), size);

		let map = self.map.entry(decls.clone()).or_default();
		if let Some(cache_entry) = map.get(&query) {
			match cache_entry {
				Count(0) => {
					return (0, None);
				}
				Explicit(explicit) => {
					let len = explicit.len();
					let id = (random() * len as f64) as usize;

					return (explicit.len(), Some(explicit[id].clone()));
				}
				_ => (),
			}
		}

		let mut explicit = Vec::with_capacity(Self::MAX_IN_MEM);

		let mut search = search(lang, decls.clone(), ty, size);

		while explicit.len() < Self::MAX_IN_MEM {
			if let Some(next) = search.next() {
				explicit.push(next);
			} else if !explicit.is_empty() {
				let len = explicit.len();
				let id = (random() * len as f64) as usize;

				map.entry(query)
					.or_insert_with(|| Explicit(explicit.clone()));

				return (len, Some(explicit.swap_remove(id)));
			} else {
				map.entry(query).or_insert(Count(0));
				return (0, None);
			}
		}

		let (rest_count, selected) = reservoir_sample(search);

		if selected.is_none() {
			let len = explicit.len();
			let id = (random() * len as f64) as usize;

			map.entry(query)
				.or_insert_with(|| Explicit(explicit.clone()));

			return (len, Some(explicit.swap_remove(id)));
		}

		let total_count = Self::MAX_IN_MEM + rest_count;
		let prob = rest_count as f64 / total_count as f64;

		map.entry(query).or_insert(Count(total_count));

		if with_probability(prob) {
			let len = explicit.len();
			let id = (random() * len as f64) as usize;
			return (total_count, Some(explicit.swap_remove(id)));
		}

		(total_count, selected)
	}

	pub fn query_count(&mut self, lang: &L, mut decls: VarsVec, ty: &Type, size: usize) -> usize {
		use CacheEntry::*;
		let query = (ty.clone(), size);

		decls.sort();

		let map = self.map.entry(decls.clone()).or_default();

		if let Some(entry) = map.get(&query) {
			return match entry {
				Count(count) => *count,
				Explicit(v) => v.len(),
			};
		}

		let count = search(lang, decls, ty, size).count();

		map.insert(query, Count(count));

		count
	}
}

impl<L: Language> Default for SizeCache<L> {
	fn default() -> Self {
		Self {
			map: Default::default(),
		}
	}
}
