use super::*;

use rustc_hash::FxHashMap as HashMap;

const CACHE_SIZE: usize = 4;

type Search = (Rc<Type>, usize);
type Analyzed = (Term, Analysis);
type PathDict = HashMap<Search, SearchResult>;
type SemanticDict = HashMap<Semantics, (Term, usize)>;

#[derive(Debug, Default, Clone)]
pub enum SearchResult {
	#[default]
	Unknown,
	Inhabited {
		cache: Vec<Analyzed>,
		state: Option<Box<Node>>,
	},
	Empty,
}

pub struct Cache {
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

	pub fn prune(&self, targ: &Rc<Type>, size: usize) -> &SearchResult {
		let search = (targ.clone(), size);

		self.active().get(&search).unwrap_or(&Unknown)
	}

	pub fn prune_arg(&self, targ: &Rc<Type>, l_ty: &Rc<Type>, size: usize) -> SearchResult {
		fn core(dict: &PathDict, targ: &Rc<Type>, l_ty: &Rc<Type>, size: usize) -> SearchResult {
			let done = l_ty == targ;

			if size == 0 && done {
				return SearchResult::LARGE;
			}

			if size == 0 || done {
				return Empty;
			}

			let Type::Fun(arg, ret) = &**l_ty else {
				unreachable!()
			};


			let mut res = Empty;
			for n in 1..size {
				let search = (arg.clone(), n);
				let arg_res = dict.get(&search).unwrap_or(&Unknown).clone();

				if arg_res.empty() {
					continue;
				}

				let rest = core(dict, targ, ret, size - n - 1);

				if arg_res.unknown() && !rest.empty() {
					res = Unknown;
					continue;
				}

				if arg_res.inhabited() && rest.inhabited() {
					res = SearchResult::LARGE;
					break;
				}
			}

			res
		}

		core(self.active(), targ, l_ty, size)
	}

	pub fn begin_search(&mut self, targ: &Rc<Type>, size: usize) {
		let search = (targ.clone(), size);

		self.active_mut().entry(search).or_insert(Unknown);
	}

	pub fn yield_term(
		&mut self,
		targ: &Rc<Type>,
		size: usize,
		node: Option<&Node>,
		term: Term,
		analysis: Analysis,
	) -> Option<Term> {
		use Analysis::*;
		match &analysis {
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
			.or_insert(Unknown)
			.log(node, &term, analysis);

		Some(term)
	}

	pub fn end_search(&mut self, search: Search) {
		/* println!("Cache:");
		for ((t, s), r) in self.active() {
			print!("\t({}, {}) -> ", t, s);
			match r {
				Small(v) => {
					print!("[");
					for (t, _) in v {
						print!("{},", t);
					}
					print!("]");
				}
				_ => print!("{:?}", r),
			}
			println!();
		} */
		
		let result = self.active_mut().get_mut(&search).unwrap();

		if result.unknown() {
			*result = Empty;
		}
	}

	fn active(&self) -> &PathDict {
		self.paths.last().unwrap()
	}

	fn active_mut(&mut self) -> &mut PathDict {
		self.paths.last_mut().unwrap()
	}
}

impl SearchResult {
	pub const LARGE: Self = Inhabited {
		cache: vec![],
		state: None,
	};

	//Add to space
	pub fn log(&mut self, node: Option<&Node>, term: &Term, analysis: Analysis) {
		match self {
			Unknown if CACHE_SIZE != 0 => {
				*self = Inhabited {
					cache: vec![(term.clone(), analysis.clone())],
					state: node.map(|n| n.clone().into()),
				}
			}
			Inhabited { cache, state } if !cache.is_empty() && cache.len() < CACHE_SIZE => {
				cache.push((term.clone(), analysis.clone()));
				let new_node = node.map(|n| Box::new(n.clone()));
				if let Some(node) = new_node {
					*state = Some(node);
				}
			}
			Inhabited { cache, state } if cache.len() == CACHE_SIZE => {
				*cache = vec![];
				let new_node = node.map(|n| Box::new(n.clone()));
				if let Some(node) = new_node {
					*state = Some(node);
				}
			}
			_ => (),
		}
	}

	pub fn unknown(&self) -> bool {
		matches!(self, Unknown)
	}

	pub fn empty(&self) -> bool {
		matches!(self, Empty)
	}

	pub fn inhabited(&self) -> bool {
		matches!(self, Inhabited { .. })
	}
}
