use super::*;

use rustc_hash::FxHashSet as HashSet;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
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

	pub fn intro_var(&mut self, is_new: bool) {
		self.empties_stack.push(Default::default());
	}

	pub fn elim_var(&mut self) {
		self.empties_stack.pop();
	}

	pub fn prune(&self, node: &SearchNode) -> bool {
		self.active_cache()
			.contains(&(node.targ.clone(), node.size))
	}

	pub fn begin_search(&mut self, node: &SearchNode) {
		self.searches
			.push((node.targ.clone(), node.size, SearchResult::NotFound));
	}

	pub fn try_yield(&mut self, term: Term) -> Option<Term> {
		for (_, size, res) in self.searches.iter_mut() {
			if *size == term.size() {
				*res = SearchResult::Inhabited;
			}
		}

		Some(term)
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
