use super::*;
use std::rc::Rc;

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
		apps: Stack<Term>,
		app_state: Option<Box<Node>>,
		app_ty: Rc<Type>,
		arg_state: Option<Box<Node>>,
		done: bool,
	},
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
	pub fn next(&mut self, search_ctxt: &mut SearchContext) -> Option<Term> {
		use Node::*;
		loop {
			match self {
				All {
					targ,
					size,
					phase,
					state,
				} => {
					if *size == 0 {
						return None;
					}

					if let Some(curr_state) = state {
						match curr_state.next(search_ctxt) {
							Some(term) => return Some(term),
							None => *state = None,
						};
					}

					let size = *size;

					use AllPhase::*;
					match phase {
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
						return None;
					};

					let ident = *ident.get_or_insert_with(|| search_ctxt.vgen.small_var());

					if let Some(curr_state) = state {
						return match curr_state.next(search_ctxt) {
							Some(term) => Some(Term::Lam(ident, term.into())),
							None => {
								search_ctxt.args.pop().unwrap();
								search_ctxt.vgen.freshen(ident);
								None
							}
						};
					};

					search_ctxt.args.push((ident, arg.clone()));

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
							return Some(Term::Var(var));
						} else {
							continue;
						}
					}

					*state = Some(Box::new(Arg {
						targ: targ.clone(),
						size: size - 1,
						apps: Stack::one(Term::Var(var)),
						app_ty: v_ty,
						app_state: None,
						arg_state: None,
						done: false,
					}));
				}

				Arg {
					targ,
					size,
					arg_state,
					app_state,
					apps,
					app_ty,
					done,
				} => {
					if *done {
						return None;
					}

					if let Some(curr_state) = app_state {
						match curr_state.next(search_ctxt) {
							Some(term) => return Some(term),
							None => *app_state = None,
						};
					};

					let size = *size;
					if size == 1 {
						return None;
					}

					if size == 0 && targ == app_ty {
						*done = true;
						return Some(apps.build_term());
					} else if size == 0 || targ == app_ty {
						return None;
					}

					let Type::Fun(arg_ty, ret_ty) = &**app_ty else {
						unreachable!()
					};

					let arg_state = arg_state.get_or_insert_with(|| {
						// If applying one arg yields target type, we skip straight to
						// the largest possible arg. Otherwise start searching args of
						// all sizes, starting from 1.
						let arg_size = if ret_ty == targ { size - 1 } else { 1 };

						Box::new(All {
							targ: arg_ty.clone(),
							size: arg_size,
							state: None,
							phase: AllPhase::START,
						})
					});

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
								*arg_size += 1;
								*phase = AllPhase::START;
								*state = None;

								if *arg_size >= size {
									return None;
								}
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
						done: false,
					}))
				}
			}
		}
	}
}
