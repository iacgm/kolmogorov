use super::*;

use std::rc::Rc;
use SearchResult::*;

#[derive(Clone, Debug)]
pub(super) enum Node<L: Language> {
    All {
        targ: Rc<Type>,
        size: usize,
        phase: AllPhase,
        state: Option<Box<Node<L>>>,
    },
    Abs {
        targ: Rc<Type>,
        size: usize,
        ident: Option<Identifier>,
        state: Option<Box<Node<L>>>,
    },
    Var {
        targ: Rc<Type>,
        size: usize,
        vars: VarsVec,
        state: Option<Box<Node<L>>>,
    },
    Arg {
        targ: Rc<Type>,
        size: usize,
        l_ty: Rc<Type>,
        left: Thunk,
        left_analysis: Analysis<L>,
        res: SearchResult<L>,
        state: Option<Box<Node<L>>>,
        arg_state: Option<Box<Node<L>>>,
    },
    Cached {
        list: Vec<(Term, Analysis<L>)>,
        next: Box<Node<L>>,
    },
    Nil,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AllPhase {
    CacheCheck,
    Application,
    Abstraction,
    Completed,
    CacheCompleted,
}

impl AllPhase {
    pub const START: Self = Self::CacheCheck;
}

impl<L: Language> Node<L> {
    pub fn next(
        &mut self,
        search_ctxt: &mut SearchContext<L>,
    ) -> Option<(Term, Analysis<L>)> {
        use Node::*;
        loop {
            #[cfg(feature = "fulltrace")]
            {
                println!("============================");
                println!("{}", &self);
            }

            match self {
                Cached { list, next } => match list.pop() {
                    Some(x) => return Some(x),
                    None => {
                        let next = std::mem::replace(next.as_mut(), Nil);
                        *self = next;
                        self.late_start(search_ctxt);
                        continue;
                    }
                },
                All {
                    targ,
                    size,
                    phase,
                    state,
                } => {
                    let size = *size;

                    if size == 0 {
                        return None;
                    }

                    if let Some(curr_state) = state {
                        match curr_state.next(search_ctxt) {
                            Some((term, analysis)) => {
                                let this = All {
                                    targ: targ.clone(),
                                    size,
                                    phase: *phase,
                                    state: state.clone(),
                                };

                                if let Some(term) =
                                    search_ctxt.cache.yield_term(
                                        targ,
                                        size,
                                        Some(&this),
                                        term,
                                        analysis.clone(),
                                    )
                                {
                                    return Some((term, analysis));
                                } else {
                                    continue;
                                }
                            }
                            None => *state = None,
                        };
                    }

                    use AllPhase::*;
                    match phase {
                        CacheCheck => {
                            match search_ctxt.cache.prune(targ, size) {
                                Inhabited {
                                    cache,
                                    state: cache_state,
                                    ..
                                } if !cache.is_empty() => {
                                    *phase = CacheCompleted;
                                    *state = Some(Box::new(Cached {
                                        list: cache.clone(),
                                        next: cache_state
                                            .clone()
                                            .unwrap_or(Nil.into()),
                                    }));
                                    continue;
                                }
                                Empty => return None,
                                _ => *phase = Application,
                            }
                            search_ctxt.cache.begin_search(targ, size);
                            continue;
                        }
                        Application => {
                            *phase = Abstraction;
                            *state = Some(Box::new(Var {
                                targ: targ.clone(),
                                size,
                                state: None,
                                vars: search_ctxt.vars_producing(targ),
                            }))
                        }
                        Abstraction => {
                            *phase = Completed;
                            *state = Some(Box::new(Abs {
                                targ: targ.clone(),
                                ident: None,
                                size,
                                state: None,
                            }))
                        }
                        Completed => {
                            let search = (targ.clone(), size);
                            search_ctxt.cache.end_search(search);
                            return None;
                        }
                        CacheCompleted => {
                            return None;
                        }
                    };
                }

                Abs {
                    targ,
                    size,
                    ident,
                    state,
                } => {
                    let Type::Fun(arg, ret) = &**targ else {
                        *self = Nil;
                        return None;
                    };

                    let ident = *ident
                        .get_or_insert_with(|| search_ctxt.vgen.small_var());

                    if let Some(curr_state) = state {
                        return match curr_state.next(search_ctxt) {
                            Some((term, analysis)) => {
                                let term = Term::Lam(ident, term.into());

                                let analysis = search_ctxt
                                    .lang
                                    .slam(ident, analysis, targ);
                                Some((term, analysis))
                            }
                            None => {
                                search_ctxt.args.pop().unwrap();
                                search_ctxt.vgen.freshen(ident);
                                search_ctxt.cache.elim_var();
                                None
                            }
                        };
                    };

                    search_ctxt.args.push((ident, arg.clone()));

                    let is_new = !search_ctxt.contains_var_of_type(arg);
                    search_ctxt.cache.intro_var(is_new);

                    *state = Some(Box::new(All {
                        targ: ret.clone(),
                        size: *size - 1,
                        state: None,
                        phase: AllPhase::START,
                    }));
                }

                Var {
                    targ,
                    size,
                    vars,
                    state,
                } => {
                    if let Some(curr_state) = state {
                        match curr_state.next(search_ctxt) {
                            Some(term) => return Some(term),
                            None => *state = None,
                        };
                    }

                    let (var, v_ty) = vars.pop()?;
                    let size = *size;

                    if size == 1 {
                        if v_ty == *targ {
                            return Some((
                                Term::Var(var),
                                search_ctxt.lang.svar(var, targ),
                            ));
                        } else {
                            continue;
                        }
                    }

                    let analysis = search_ctxt.lang.svar(var, targ);

                    *state = Some(Box::new(Arg {
                        targ: targ.clone(),
                        size: size - 1,
                        l_ty: v_ty,
                        left: Term::Var(var).into(),
                        left_analysis: analysis,
                        state: None,
                        arg_state: None,
                        res: Unknown,
                    }));
                }

                Arg {
                    targ,
                    size,
                    l_ty,
                    left,
                    left_analysis,
                    state,
                    arg_state,
                    res,
                } => {
                    if left_analysis.malformed() {
                        *self = Nil;
                        return None;
                    }

                    if let Some(curr_state) = state {
                        match curr_state.next(search_ctxt) {
                            Some(term) => return Some(term),
                            None => *state = None,
                        };
                    };

                    let size = *size;
                    if size == 1 {
                        *self = Nil;
                        return None;
                    }

                    if size == 0 && targ == l_ty {
                        let Arg {
                            left,
                            left_analysis,
                            ..
                        } = std::mem::replace(self, Nil)
                        else {
                            unreachable!()
                        };

                        return Some((
                            left.borrow().clone(),
                            left_analysis.clone(),
                        ));
                    } else if size == 0 || targ == l_ty {
                        *self = Nil;
                        return None;
                    }

                    let Type::Fun(arg_ty, ret_ty) = &**l_ty else {
                        unreachable!()
                    };

                    if res.unknown() {
                        *res = search_ctxt.cache.prune_arg(targ, l_ty, size);

                        if res.empty() {
                            self.early_exit(search_ctxt);
                            *self = Nil;
                            return None;
                        }
                    }

                    let arg_state = match arg_state {
                        Some(arg_state) => arg_state,
                        None => {
                            // If applying one arg yields target type, we skip straight to
                            // the largest possible arg. Otherwise start searching args of
                            // all sizes, starting from 1.
                            let arg_size =
                                if ret_ty == targ { size - 1 } else { 1 };

                            *arg_state = Some(Box::new(All {
                                targ: arg_ty.clone(),
                                size: arg_size,
                                state: None,
                                phase: AllPhase::START,
                            }));

                            arg_state.as_mut().unwrap()
                        }
                    };

                    // Get the next arg, trying every size until we generate one.
                    let ((arg, arg_analysis), arg_size) = loop {
                        if let Some(arg) = arg_state.next(search_ctxt) {
                            let size = arg.0.size();
                            break (arg, size);
                        }

                        let All {
                            size: arg_size,
                            state,
                            phase,
                            ..
                        } = &mut **arg_state
                        else {
                            unreachable!()
                        };

                        if *arg_size == size - 1 {
                            *self = Nil;
                            return None;
                        }

                        *arg_size += 1;
                        *phase = AllPhase::START;
                        *state = None;
                    };

                    let analysis = search_ctxt.lang.sapp(
                        left_analysis.clone(),
                        arg_analysis,
                        targ,
                    );
                    let left = Term::App(left.clone(), arg.into());

                    if let Some(term) = search_ctxt.cache.yield_term(
                        ret_ty,
                        left.size(),
                        None,
                        left,
                        analysis.clone(),
                    ) {
                        *state = Some(Box::new(Arg {
                            targ: targ.clone(),
                            size: size - arg_size - 1,
                            l_ty: ret_ty.clone(),
                            left: term.into(),
                            left_analysis: analysis,
                            state: None,
                            arg_state: None,
                            res: Unknown,
                        }))
                    }
                }
                Nil => return None,
            }
        }
    }

    // Used to initialize a search at an arbitrary node (to skip ahead in caching)
    pub fn late_start(&self, search_ctxt: &mut SearchContext<L>) {
        use Node::*;
        match self {
            All {
                targ, size, state, ..
            } => {
                search_ctxt.cache.begin_search(targ, *size);
                if let Some(state) = state.as_ref() {
                    state.late_start(search_ctxt);
                }
            }
            Abs {
                state: Some(state), ..
            }
            | Var {
                state: Some(state), ..
            } => state.late_start(search_ctxt),
            Arg {
                state, arg_state, ..
            } => {
                if let Some(state) = arg_state.as_ref() {
                    state.late_start(search_ctxt);
                }
                if let Some(state) = state.as_ref() {
                    state.late_start(search_ctxt);
                }
            }
            _ => {}
        }
    }

    //Used to end a search early (if pruning indicates emptiness)
    pub fn early_exit(&mut self, search_ctxt: &mut SearchContext<L>) {
        use Node::*;
        match self {
            All {
                state, targ, size, ..
            } => {
                if let Some(state) = state.take().as_mut() {
                    state.early_exit(search_ctxt);
                }

                let search = (targ.clone(), *size);
                search_ctxt.cache.end_search(search);
            }
            Abs {
                state: Some(state), ..
            }
            | Var {
                state: Some(state), ..
            } => {
                state.early_exit(search_ctxt);
            }
            Arg {
                state, arg_state, ..
            } => {
                if let Some(state) = state.take().as_mut() {
                    state.early_exit(search_ctxt);
                }
                if let Some(state) = arg_state.take().as_mut() {
                    state.early_exit(search_ctxt);
                }
            }
            _ => {}
        }
    }
}

use std::fmt::*;
impl<L: Language + Debug> Display for Node<L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        static mut DISP_DEPTH: usize = 0;
        let indent = 4 * unsafe {
            DISP_DEPTH += 1;
            DISP_DEPTH - 1
        };

        use Node::*;
        let out = match self {
            Cached { list, next } => {
                write!(f, "{:indent$}Cached: [", "")?;
                for (t, a) in list {
                    write!(f, "({} ≈ {}),", t, a)?;
                }
                writeln!(f, "]")?;
                write!(
                    f,
                    "\n{:indent$}next_state:\n{}",
                    "",
                    next,
                    indent = indent
                )
            }
            All {
                targ,
                size,
                phase,
                state,
            } => {
                write!(
                    f,
                    "{:indent$}All {}, {}, {}: ",
                    "",
                    targ,
                    size,
                    phase,
                    indent = indent
                )?;
                if let Some(state) = state {
                    write!(f, "\n{}", state)
                } else {
                    writeln!(f, "Nil")
                }
            }
            Abs {
                targ,
                size,
                ident,
                state,
            } => {
                write!(
                    f,
                    "{:indent$}Abs {}, {}, {:?}: ",
                    "",
                    targ,
                    size,
                    ident,
                    indent = indent
                )?;
                if let Some(state) = state {
                    write!(f, "\n{}", state)
                } else {
                    writeln!(f, "Nil")
                }
            }
            Var {
                targ,
                size,
                vars,
                state,
            } => {
                write!(
                    f,
                    "{:indent$}Var {}, {}, {:?}: ",
                    "",
                    targ,
                    size,
                    vars,
                    indent = indent
                )?;
                if let Some(state) = state {
                    write!(f, "\n{}", state)
                } else {
                    writeln!(f, "Nil")
                }
            }
            Arg {
                targ,
                size,
                l_ty,
                left,
                left_analysis,
                res,
                state,
                arg_state,
            } => {
                write!(
                    f,
                    "{:indent$}Arg {}, {}, {:?}, left = {} ≈ {}, {}:",
                    "",
                    targ,
                    size,
                    res,
                    left.borrow(),
                    left_analysis,
                    l_ty,
                    indent = indent
                )?;
                if let Some(state) = state {
                    write!(
                        f,
                        "\n{:indent$}state:\n{}",
                        "",
                        state,
                        indent = indent
                    )?;
                } else {
                    writeln!(f, "Nil ")?;
                }
                if let Some(state) = arg_state {
                    write!(
                        f,
                        "\n{:indent$}arg_state:\n{}",
                        "",
                        state,
                        indent = indent
                    )
                } else {
                    writeln!(f, "Nil")
                }
            }
            Nil => writeln!(f, "{:indent$}Nil", "", indent = indent),
        };

        unsafe { DISP_DEPTH -= 1 };
        out
    }
}

impl Display for AllPhase {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self)
    }
}
