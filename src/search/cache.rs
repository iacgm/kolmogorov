use super::*;

use rustc_hash::FxHashMap as HashMap;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SearchResult {
	#[default]
	Unknown,
	Inhabited,
	Uninhabited,
}

type Search = (Rc<Type>, usize);
type PathDict = HashMap<Search, SearchResult>;
type SemanticDict = HashMap<Term, (Term, usize)>;

pub struct Cache {
	searches: Vec<Search>,
	paths: Vec<PathDict>,
	// Minimal sizes of representations of constants
	consts: Vec<SemanticDict>,
	// Top element indicates whether pathdict should be popped.
	pops: Vec<bool>,
}

use SearchResult::*;
impl Cache {
	pub fn new() -> Self {
		Self {
			searches: vec![],
			paths: vec![Default::default()],
			consts: vec![Default::default()],
			pops: vec![],
		}
	}

	pub fn intro_var(&mut self, is_new: bool) {
		if is_new {
			self.paths.push(Default::default());
			self.consts.push(Default::default());
		}
		self.pops.push(is_new);
	}

	pub fn elim_var(&mut self) {
		if self.pops.pop().unwrap() {
			self.paths.pop();
			self.consts.pop();
		}
	}

	pub fn prune(&self, targ: &Rc<Type>, size: usize) -> bool {
		let search = (targ.clone(), size);

		self.active().get(&search) == Some(&Uninhabited)
	}

	pub fn prune_arg(&self, targ: &Rc<Type>, l_ty: &Rc<Type>, size: usize) -> SearchResult {
		fn core(dict: &PathDict, targ: &Rc<Type>, l_ty: &Rc<Type>, size: usize) -> SearchResult {
			let last = l_ty == targ;

			if size == 0 && last {
				return Inhabited;
			}

			if size == 0 || last {
				return Uninhabited;
			}

			let Type::Fun(arg, ret) = &**l_ty else {
				unreachable!()
			};

			let mut res = Uninhabited;
			for n in 1..size {
				let search = (arg.clone(), n);
				let arg_res = *dict.get(&search).unwrap_or(&Unknown);

				if arg_res == Uninhabited {
					continue;
				}

				let rest = core(dict, targ, ret, size - n - 1);

				if arg_res == Unknown && matches!(rest, Unknown | Inhabited) {
					res = Unknown;
					continue;
				}

				if arg_res == Inhabited && rest == Inhabited {
					res = Inhabited;
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
		ctxt: &Context,
		term: Term,
	) -> Option<Term> {
		let mut canon = term.deep_clone();

		let canonized = (ctxt.canonizer)(&mut canon);

		if canonized {
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

		let search = (targ.clone(), size);

		self.active_mut()
			.entry(search)
			.and_modify(|r| *r = Inhabited)
			.or_insert(Inhabited);

		Some(term)
	}

	pub fn end_search(&mut self) {
		let search = self.searches.pop().unwrap();

		let result = self.active_mut().get_mut(&search).unwrap();

		if *result == Unknown {
			*result = Uninhabited;
		}
	}

	fn active(&self) -> &PathDict {
		self.paths.last().unwrap()
	}

	fn active_mut(&mut self) -> &mut PathDict {
		self.paths.last_mut().unwrap()
	}
}
