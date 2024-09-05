use super::*;
use std::rc::Rc;
use SearchResult::*;

#[derive(Debug)]
pub(super) enum Node {
	All {
		targ: Rc<Type>,
		size: usize,
		state: Option<Box<Node>>,
		phase: AllPhase,
	},
	Abs {
		targ: Rc<Type>,
		size: usize,
		state: Option<Box<Node>>,
		ident: Option<Identifier>,
	},
	Var {
		targ: Rc<Type>,
		size: usize,
		state: Option<Box<Node>>,
		vars: VarsVec,
	},
	Arg {
		targ: Rc<Type>,
		size: usize,
		apps: Stack<NTerm>,
		app_state: Option<Box<Node>>,
		app_ty: Rc<Type>,
		arg_state: Option<Box<Node>>,
		res: SearchResult,
	},
	Nil,
}

#[derive(Debug)]
pub(super) enum AllPhase {
	Application,
	Abstraction,
	Completed,
}

impl AllPhase {
	pub const START: Self = Self::Application;
}

impl Node {
	pub fn next(&mut self, search_ctxt: &mut SearchContext) -> Option<NTerm> {
		use Node::*;
		loop {
			//dbg!(&self);
			match self {
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
							Some(term) => {
								if let Some(term) = search_ctxt.cache.yield_term(
									targ,
									size,
									&search_ctxt.ctxt,
									term,
								) {
									return Some(term);
								} else {
									continue;
								}
							}
							None => *state = None,
						};
					}

					use AllPhase::*;
					match phase {
						Application => {
							if search_ctxt.cache.prune(targ, size) {
								return None;
							}

							search_ctxt.cache.begin_search(targ, size);

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
							search_ctxt.cache.end_search();
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

					let ident = *ident.get_or_insert_with(|| search_ctxt.vgen.small_var());

					if let Some(curr_state) = state {
						return match curr_state.next(search_ctxt) {
							Some(term) => Some(NTerm::Lam(ident, term.into())),
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
							return Some(NTerm::Var(var));
						} else {
							continue;
						}
					}

					*state = Some(Box::new(Arg {
						targ: targ.clone(),
						size: size - 1,
						apps: Stack::one(NTerm::Var(var)),
						app_ty: v_ty,
						app_state: None,
						arg_state: None,
						res: Unknown,
					}));
				}

				Arg {
					targ,
					size,
					arg_state,
					app_state,
					apps,
					app_ty,
					res,
				} => {
					if let Some(curr_state) = app_state {
						match curr_state.next(search_ctxt) {
							Some(term) => return Some(term),
							None => *app_state = None,
						};
					};

					let size = *size;
					if size == 1 {
						*self = Nil;
						return None;
					}

					if size == 0 && targ == app_ty {
						let Arg { apps, .. } = std::mem::replace(self, Nil) else {
							unreachable!()
						};

						return Some(apps.build_term());
					} else if size == 0 || targ == app_ty {
						*self = Nil;
						return None;
					}

					let Type::Fun(arg_ty, ret_ty) = &**app_ty else {
						unreachable!()
					};

					if *res == Unknown {
						*res = search_ctxt.cache.prune_arg(targ, app_ty, size);

						if *res == Uninhabited {
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
							let arg_size = if ret_ty == targ { size - 1 } else { 1 };

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
					let (arg, arg_size) = loop {
						let next_arg = arg_state.next(search_ctxt);

						let All {
							size: arg_size,
							state,
							phase,
							..
						} = &mut **arg_state
						else {
							unreachable!()
						};

						match next_arg {
							Some(arg) => break (arg, *arg_size),
							None => {
								if *arg_size == size - 1 {
									*self = Nil;
									return None;
								}

								*arg_size += 1;
								*phase = AllPhase::START;
								*state = None;
							}
						}
					};

					*app_state = Some(Box::new(Arg {
						targ: targ.clone(),
						size: size - arg_size - 1,
						apps: apps.cons(arg),
						app_state: None,
						app_ty: ret_ty.clone(),
						arg_state: None,
						res: Unknown,
					}))
				}
				Nil => return None,
			}
		}
	}

	pub fn early_exit(&mut self, search_ctxt: &mut SearchContext) {
		use Node::*;
		match self {
			All { state, .. } => {
				if let Some(state) = state.take().as_mut() {
					state.early_exit(search_ctxt);
				}
				search_ctxt.cache.end_search();
			}
			Abs { state, .. } | Var { state, .. } => {
				if let Some(state) = state.take().as_mut() {
					state.early_exit(search_ctxt);
				}
			}
			Arg {
				app_state,
				arg_state,
				..
			} => {
				if let Some(state) = app_state.take().as_mut() {
					state.early_exit(search_ctxt);
				}
				if let Some(state) = arg_state.take().as_mut() {
					state.early_exit(search_ctxt);
				}
			}
			Nil => {}
		}
	}
}
