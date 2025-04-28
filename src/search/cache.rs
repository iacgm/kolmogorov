use super::*;

use rustc_hash::FxHashMap as HashMap;

const CACHE_SIZE: usize = 8;

type Search = (Rc<Type>, usize);
type Analyzed<L> = (Term, Analysis<L>);
type PathDict<L> = HashMap<Search, SearchResult<L>>;
type SemanticDict<L> =
    HashMap<(<L as Language>::Semantics, Type), (Term, usize)>;

#[derive(Debug, Default, Clone)]
pub enum SearchResult<L: Language> {
    #[default]
    Unknown,
    Inhabited {
        cache: Vec<Analyzed<L>>,
        state: Option<Box<Node<L>>>,
    },
    Empty,
}

pub struct Cache<L: Language> {
    paths: Vec<PathDict<L>>,
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

    pub fn prune(&self, targ: &Rc<Type>, size: usize) -> &SearchResult<L> {
        let search = (targ.clone(), size);

        self.active().get(&search).unwrap_or(&Unknown)
    }

    pub fn prune_arg(
        &self,
        targ: &Rc<Type>,
        l_ty: &Rc<Type>,
        size: usize,
    ) -> SearchResult<L> {
        fn core<L: Language>(
            dict: &PathDict<L>,
            targ: &Rc<Type>,
            l_ty: &Rc<Type>,
            size: usize,
        ) -> SearchResult<L> {
            let done = l_ty == targ;

            if size == 0 && done {
                return SearchResult::large();
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

                if (arg_res.unknown() && !rest.empty())
                    || (arg_res.inhabited() && rest.unknown())
                {
                    res = Unknown;
                    continue;
                }

                if arg_res.inhabited() && rest.inhabited() {
                    res = SearchResult::large();
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
        node: Option<&Node<L>>,
        term: Term,
        analysis: Analysis<L>,
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
                        if *m_size < size
                            || (*m_size == size && &term != minimal)
                        {
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

    pub fn active(&self) -> &PathDict<L> {
        self.paths.last().unwrap()
    }

    fn active_mut(&mut self) -> &mut PathDict<L> {
        self.paths.last_mut().unwrap()
    }
}

impl<L: Language> SearchResult<L> {
    pub fn large() -> Self {
        Inhabited {
            cache: vec![],
            state: None,
        }
    }

    //Add to space
    pub fn log(
        &mut self,
        node: Option<&Node<L>>,
        term: &Term,
        analysis: Analysis<L>,
    ) {
        match self {
            Unknown if CACHE_SIZE != 0 => {
                *self = Inhabited {
                    cache: vec![(term.clone(), analysis.clone())],
                    state: node.map(|n| n.clone().into()),
                }
            }
            Inhabited { cache, .. }
                if cache.len() >= CACHE_SIZE
                    || cache.iter().any(|t| &t.0 == term) => {}
            Inhabited { cache, state }
                if (1..CACHE_SIZE).contains(&cache.len()) =>
            {
                cache.push((term.clone(), analysis.clone()));
                if node.is_some() {
                    *state = node.map(|n| Box::new(n.clone()));
                }
            }
            Inhabited { .. } => {}
            _ => {
                unreachable!()
            }
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
