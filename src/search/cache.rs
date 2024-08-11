use super::*;

use rustc_hash::FxHashSet as HashSet;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum SearchResult {
	#[default]
	NotFound,
	Inhabited,
}

// All empty (type, size)
type EmptyPaths = HashSet<(Rc<Type>, usize)>;

pub struct Cache {
	// Hashmaps representing paths known to be empty
	empties_stack: Vec<EmptyPaths>,
	//Currently open, uninhabited searches (by type & size)
	searches: Vec<(Rc<Type>, usize, SearchResult)>,
}

impl Cache {
	pub fn new() -> Self {
		Self {
			empties_stack: vec![Default::default()],
			searches: vec![],
		}
	}

	pub fn intro_var(&mut self, _is_new: bool) {
		self.empties_stack.push(Default::default());
	}

	pub fn elim_var(&mut self) {
		self.empties_stack.pop();
	}

	pub fn prune(&self, targ: &Rc<Type>, size: usize) -> bool {
		self.active_cache().contains(&(targ.clone(), size))
	}

	pub fn prune_arg(&self, targ: &Rc<Type>, l_ty: &Rc<Type>, size: usize) -> bool {
		fn core(cache: &Cache, targ: &Rc<Type>, l_ty: &Rc<Type>, size: usize) -> bool {
			let last = l_ty == targ;

			if size == 0 || last {
				return !(size == 0 && last);
			}

			let Type::Fun(arg, ret) = &**l_ty else {
				unreachable!()
			};

			(1..size).all(|n| cache.prune(arg, n) || core(cache, targ, ret, size - n - 1))
		}

		core(self, targ, l_ty, size)
	}

	pub fn begin_search(&mut self, targ: &Rc<Type>, size: usize) {
		self.searches
			.push((targ.clone(), size, SearchResult::NotFound));
	}

	pub fn yield_term(&mut self, _term: &Term, size: usize) {
		for (_, search_size, res) in self.searches.iter_mut() {
			if *search_size == size {
				*res = SearchResult::Inhabited;
			}
		}
	}

	pub fn end_search(&mut self) {
		let (ty, size, res) = self.searches.pop().unwrap();

		if res == SearchResult::NotFound {
			self.active_cache_mut().insert((ty, size));
		}
	}

	fn active_cache(&self) -> &EmptyPaths {
		self.empties_stack.last().unwrap()
	}

	fn active_cache_mut(&mut self) -> &mut EmptyPaths {
		self.empties_stack.last_mut().unwrap()
	}
}
