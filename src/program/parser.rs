#[macro_export]
macro_rules! term {
	($x: ident) => {
		$crate::Term::Var(stringify!($x))
	};
	([$x: expr]) => {
		$x.clone()
	};
	($x: literal) => {
		$crate::Term::Num($x)
	};
	($x:ident -> $($r:tt)+) => {
		$crate::Term::Lam(stringify!($x), term!($($r)+).into())
	};
	($x:ident $($xs:ident)+ -> $($r:tt)+) => {
		$crate::Term::Lam(stringify!($x), term!($($xs)* -> $($r)+).into())
	};
	(($($r:tt)+)) => {
		term!($($r)+)
	};
	($($r:tt)+) => {{
		let mut terms = vec![$(term!($r)),*];
		terms.reverse();
		$crate::Term::App(terms)
	}};
}

#[macro_export]
macro_rules! builtin {
	(
		$(forall $($a:ident)+)? :: $($ty:tt)=>+
		$(using [$($captured:ident),+] in)?
		|$ctx: ident, $($arg:ident),+| => $body:expr
	) => {{
		use $crate::*;
		use std::rc::Rc;

		let ty = ty!($(forall $($a)+ ::)? $($ty)=>+);

		let n_args = count!($($arg)+);

		$($(
			let $captured = $captured.clone();
		)+)?

		let func = Rc::new(move |$ctx: &mut Context, _args: &mut [Term]| {
			let rev_list!([$($arg),+]) = &mut _args[..] else {
				unreachable!()
			};

			$(
				let mut $arg = std::mem::replace($arg, $crate::Term::Num(0));
			)+

			$body
		});

		ContextEntry {
			n_args,
			ty,
			func,
			active: true,
		}
	}}
}

#[macro_export]
macro_rules! ty {
	(N) => {{
		use $crate::*;
		PolyType::from(MonoType::Int)
	}};
	(Int) => {{
		use $crate::*;
		PolyType::from(MonoType::Int)
	}};
	(Bool) => {
		ty!(forall _t :: _t => _t => _t)
	};
	($x:ident) => {{
		use $crate::*;
		PolyType::from(MonoType::Name(stringify!($x)))
	}};
	(forall $($args:ident)+ :: $($b:tt)*) => {{
		let mut poly = ty!($($b)*);
		$(
			poly.vars.insert(stringify!($args));
		)+
		poly
	}};
	([$a:tt]) => {
		ty!(forall _t :: _t => ($a => _t => _t) => _t)
	};
	($a:tt => $($b:tt)+) => {{
		let mut from = ty!($a);
		let to = ty!($($b)+);
		$crate::PolyType::func(from, to)
	}};
	(($($r:tt)+)) => {
		ty!($($r)+)
	};
}

#[macro_export]
macro_rules! rev_list {
    ([] $($reversed:ident),*) => {
        [$($reversed),*]
    };
    ([$first:ident $(, $rest:ident)*] $($reversed:ident),*) => {
        rev_list!([$($rest),*] $first $(,$reversed)*)
    };
}

#[macro_export]
macro_rules! count {
    () => { 0 };
    ($x:ident $($xs:ident)*) => { 1 + count!($($xs)*)};
}
