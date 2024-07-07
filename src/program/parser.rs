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

		let ty = poly!($(forall $($a)+)? :: $($ty)=>+);

		let n_args = count!($($arg)+);

		$($(
			let $captured = $captured.clone();
		)+)?

		let func = Rc::new(move |$ctx: &mut Dictionary, _args: &mut [Term]| {
			let rev_list!([$($arg),+]) = &mut _args[..] else {
				unreachable!()
			};

			$(
				let mut $arg = std::mem::replace($arg, $crate::Term::Num(0));
			)+

			$body
		});

		(BuiltIn {
			n_args,
			func,
		}.into(), ty)
	}}
}

#[macro_export]
macro_rules! poly {
	($(forall $($t:ident)+)? :: $($b:tt)+) => {
		$crate::PolyType {
			vars: std::collections::HashSet::from([$($(stringify!($t)),+)?]),
			mono: $crate::mono!($($b)+)
		}
	};
}

#[macro_export]
macro_rules! mono {
	(N) => {
		$crate::MonoType::Int
	};
	(Int) => {
		$crate::MonoType::Int
	};
	($x: ident) => {
		$crate::MonoType::Var(stringify!($x))
	};
	($a:tt => $($b:tt)+) => {
		$crate::MonoType::Fun(mono!($a).into(), mono!($($b)+).into())
	};
	(($($r:tt)+)) => {
		mono!($($r)+)
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
