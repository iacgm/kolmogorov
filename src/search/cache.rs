use super::*;

use rustc_hash::FxHashMap as HashMap;

type Search = (Rc<Type>, usize);
type PathDict = HashMap<Search, SearchResult>;
type SemanticDict<L> = HashMap<(<L as Language>::Semantics, Type), (Term, usize)>;

#[derive(Debug, Default, Clone)]
pub enum SearchResult {
    #[default]
    Unknown,
    Inhabited,
    Empty,
}

pub struct Cache<L: Language> {
    paths: Vec<PathDict>,
    // Minimal sizes of representations of constants
    consts: Vec<SemanticDict<L>>,
}

use SearchResult::*;
impl<L: Language> Cache<L> {
    pub fn new() -> Self {
        Self {
            paths: vec![Default::default()],
            consts: vec![Default::default()],
        }
    }

    pub fn intro_var(&mut self, _is_new: bool) {
        self.paths.push(Default::default());
        self.consts.push(Default::default());
    }

    pub fn elim_var(&mut self) {
        self.paths.pop();
        self.consts.pop();
    }

    pub fn prune(&self, targ: &Rc<Type>, size: usize) -> &SearchResult {
        let search = (targ.clone(), size);

        self.active().get(&search).unwrap_or(&Unknown)
    }

    pub fn prune_arg(&self, targ: &Rc<Type>, l_ty: &Rc<Type>, size: usize) -> SearchResult {
        fn core<L: Language>(
            dict: &PathDict,
            targ: &Rc<Type>,
            l_ty: &Rc<Type>,
            size: usize,
        ) -> SearchResult {
            let done = l_ty == targ;

            if size == 0 && done {
                return SearchResult::Inhabited;
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

                let rest = core::<L>(dict, targ, ret, size - n - 1);

                if (arg_res.unknown() && !rest.empty()) || (arg_res.inhabited() && rest.unknown()) {
                    res = Unknown;
                    continue;
                }

                if arg_res.inhabited() && rest.inhabited() {
                    res = SearchResult::Inhabited;
                    break;
                }
            }

            res
        }

        core::<L>(self.active(), targ, l_ty, size)
    }

    // Returns index of search for logging
    pub fn begin_search(&mut self, targ: &Rc<Type>, size: usize) -> usize {
        let search = (targ.clone(), size);

        self.active_mut().entry(search).or_insert(Unknown);

        self.paths.len() - 1
    }

    pub fn yield_term(
        &mut self,
        targ: &Rc<Type>,
        size: usize,
        term: Term,
        analysis: Analysis<L>,
        depth: usize,
    ) -> Option<Term> {
        use Analysis::*;
        match &analysis {
            Malformed => return None,
            Unique => (),
            Canonical(canon) => {
                let entry = self
                    .consts
                    .last_mut()
                    .unwrap()
                    .entry((canon.clone(), (**targ).clone()));

                use std::collections::hash_map::Entry::*;
                match entry {
                    Occupied(mut entry) => {
                        let (minimal, m_size) = entry.get();

                        // Need the second check because we generate the same term several times
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

        for dict in &mut self.paths[depth..] {
            let search = (targ.clone(), size);
            dict.entry(search).or_insert(Unknown).log();
        }

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

    pub fn active(&self) -> &PathDict {
        self.paths.last().unwrap()
    }

    fn active_mut(&mut self) -> &mut PathDict {
        self.paths.last_mut().unwrap()
    }
}

impl SearchResult {
    //Add to space
    pub fn log(&mut self) {
        *self = Inhabited;
    }

    pub fn unknown(&self) -> bool {
        matches!(self, Unknown)
    }

    pub fn empty(&self) -> bool {
        matches!(self, Empty)
    }

    pub fn inhabited(&self) -> bool {
        matches!(self, Inhabited)
    }
}
