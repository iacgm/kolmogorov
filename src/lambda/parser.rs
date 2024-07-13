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
		$($ty:tt)=>+
		$(using [$($captured:ident),+] in)?
		|$($arg:ident),*| => $body:expr
	) => {{
		use $crate::*;
		use std::rc::Rc;

		let ty = ty!($($ty)=>+);

		let n_args = count!($($arg)*);

		$($(
			let $captured = $captured.clone();
		)+)?

		let func = Rc::new(move |_args: &mut [Term]| {
			let rev_list!([$($arg),*]) = &mut _args[..] else {
				unreachable!()
			};

			$(
				let mut $arg = std::mem::replace($arg, $crate::Term::Num(0));
			)*

			$body
		});

		Def::from((BuiltIn {
			n_args,
			func,
		}, ty))
	}}
}

#[macro_export]
macro_rules! dict {
	{$($def:ident),*} => {
		Dictionary::new(&[$(
			(stringify!($def), $def.clone())
		),*])
	};
}

#[macro_export]
macro_rules! ty {
	(N) => {
		$crate::Type::Int
	};
	(Int) => {
		$crate::Type::Int
	};
	([$e:expr]) => {
		$e
	};
	($x: ident) => {
		$crate::Type::Var(stringify!($x))
	};
	($a:tt => $($b:tt)+) => {
		$crate::Type::Fun(ty!($a).into(), ty!($($b)+).into())
	};
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
