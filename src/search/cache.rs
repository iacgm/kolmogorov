use super::*;

use rustc_hash::FxHashMap as HashMap;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SearchResult {
	#[default]
	Unknown,
	Searching(usize),
	Exhausted(usize),
}

type Search = (Rc<Type>, usize);
type PathDict = HashMap<Search, SearchResult>;
type SemanticDict = HashMap<Semantics, (Term, usize)>;

pub struct Cache {
	searches: Vec<Search>,
	paths: Vec<PathDict>,
	// Minimal sizes of representations of constants
	consts: Vec<SemanticDict>,
}

use SearchResult::*;
impl Cache {
	pub fn new() -> Self {
		Self {
			searches: vec![],
			paths: vec![Default::default()],
			consts: vec![Default::default()],
		}
	}

	pub fn intro_var(&mut self, is_new: bool) {
		let mut new_paths = PathDict::default();

		for (search, result) in self.active() {
			match *result {
				Exhausted(0) if !is_new => {
					new_paths.insert(search.clone(), Exhausted(0));
				}
				Searching(n) | Exhausted(n) => {
					new_paths.insert(search.clone(), Searching(n));
				}
				_ => (),
			};
		}

		self.paths.push(new_paths);
		self.consts.push(Default::default());
	}

	pub fn elim_var(&mut self) {
		self.paths.pop();
		self.consts.pop();
	}

	pub fn prune(&self, targ: &Rc<Type>, size: usize) -> bool {
		let search = (targ.clone(), size);

		self.active().get(&search) == Some(&Exhausted(0))
	}

	pub fn prune_arg(&self, targ: &Rc<Type>, l_ty: &Rc<Type>, size: usize) -> SearchResult {
		fn core(dict: &PathDict, targ: &Rc<Type>, l_ty: &Rc<Type>, size: usize) -> SearchResult {
			let last = l_ty == targ;

			if size == 0 && last {
				return Exhausted(1);
			}

			if size == 0 || last {
				return Exhausted(0);
			}

			let Type::Fun(arg, ret) = &**l_ty else {
				unreachable!()
			};

			let mut res = Unknown;
			for n in 1..size {
				let search = (arg.clone(), n);
				let arg_res = *dict.get(&search).unwrap_or(&Unknown);

				if arg_res == Exhausted(0) {
					continue;
				}

				let rest = core(dict, targ, ret, size - n - 1);

				if arg_res.unknown() && rest != Exhausted(0) {
					res = Unknown;
					continue;
				}

				let (Some(n), Some(m)) = (arg_res.count(), rest.count()) else {
					continue;
				};

				if n != 0 && m != 0 {
					res.inc(n * m);
					break;
				}
			}

			res
		}

		core(self.active(), targ, l_ty, size)
	}

	pub fn begin_search(&mut self, targ: &Rc<Type>, size: usize) {
		let search = (targ.clone(), size);

		self.searches.push(search.clone());

		self.active_mut().entry(search).or_insert(Unknown);
	}

	pub fn yield_term(
		&mut self,
		targ: &Rc<Type>,
		size: usize,
		term: Term,
		analysis: Analysis,
	) -> Option<Term> {
		use Analysis::*;
		match analysis {
			Malformed => return None,
			Unique => (),
			Canonical(canon) => {
				let entry = self.consts.last_mut().unwrap().entry(canon.clone());

				use std::collections::hash_map::Entry::*;
				match entry {
					Occupied(mut entry) => {
						let (minimal, m_size) = entry.get();
						if *m_size < size || (*m_size == size && &term != minimal) {
							return None;
						} else {
							*entry.get_mut() = (term.clone(), size);
						}
					}
					e => {
						e.or_insert((term.clone(), size));
					}
				};
			}
		}

		let search = (targ.clone(), size);

		self.active_mut()
			.entry(search)
			.and_modify(|r| r.inc(1))
			.or_insert(Unknown);

		Some(term)
	}

	pub fn end_search(&mut self) {
		let search = self.searches.pop().unwrap();

		let result = self.active_mut().get_mut(&search).unwrap();

		result.finish();
	}

	fn active(&self) -> &PathDict {
		self.paths.last().unwrap()
	}

	fn active_mut(&mut self) -> &mut PathDict {
		self.paths.last_mut().unwrap()
	}
}

impl SearchResult {
	pub fn inc(&mut self, n: usize) {
		match self {
			Unknown => *self = Searching(n),
			Searching(m) => *m += n,
			_ => (),
		};
	}

	pub fn finish(&mut self) {
		match self {
			Unknown => *self = Exhausted(0),
			Searching(n) => *self = Exhausted(*n),
			_ => (),
		}
	}

	pub fn unknown(&self) -> bool {
		matches!(self, Unknown | Searching(0))
	}

	pub fn count(&self) -> Option<usize> {
		match *self {
			Searching(n) | Exhausted(n) if n != 0 => Some(n),
			_ => None,
		}
	}

	pub fn inhabited(&self) -> bool {
		matches!(self, Searching(n) | Exhausted(n) if *n != 0)
	}
}
