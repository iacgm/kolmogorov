#[macro_export]
macro_rules! term {
	(_) => {
		$crate::Term::Var($crate::Identifier::Name("_"))
	};
	($x: ident) => {
		$crate::Term::Var($crate::Identifier::Name(stringify!($x)))
	};
	([$x: expr]) => {
		$x.clone()
	};
	($x: literal) => {
		$crate::Term::Val(std::rc::Rc::new($x))
	};
	($x:ident -> $($r:tt)+) => {
		$crate::Term::Lam($crate::Identifier::Name(stringify!($x)), $crate::term!($($r)+).into())
	};
	($x:ident $($xs:ident)+ -> $($r:tt)+) => {
		$crate::Term::Lam($crate::Identifier::Name(stringify!($x)), $crate::term!($($xs)* -> $($r)+).into())
	};
	(($($r:tt)+)) => {
		$crate::term!($($r)+)
	};
	($h:tt $($args:tt)+) => {{
		let mut start = $crate::term!($h);

		$(
			start = $crate::Term::App(
				start.into(),
				$crate::term!($args).into()
			);
		)+

		start
	}};
}

#[macro_export]
macro_rules! builtin {
	(
		$($ty:tt)=>+
		$(with [$($captured:ident),+] in)?
		|$($arg:ident),*| => $body:expr
	) => {{
		use $crate::*;
		use std::rc::Rc;

		let ty = ty!($($ty)=>+);

		let n_args = count!($($arg)*);

		$($(
			let $captured = $captured.clone();
		)+)?

		let func = Rc::new(move |_args: &[Thunk]| {
			let rev_list!([$($arg),*]) = &_args[..] else {
				unreachable!()
			};

			$(
				let $arg = (**$arg).borrow();
			)*

			Some($body)
		});

		BuiltIn {
			n_args,
			func,
			ty: std::rc::Rc::new(ty)
		}
	}}
}

#[macro_export]
macro_rules! context {
	{$($def:ident),*} => {{
		Context::new(
			&[$(
				($crate::Identifier::Name(stringify!($def)), $def.clone())
			),*],
		)
	}};
}

#[macro_export]
macro_rules! ty {
	([$e:expr]) => {
		$e
	};
	($x: ident) => {
		$crate::Type::Var(Identifier::from(stringify!($x)))
	};
	($a:tt => $($b:tt)+) => {
		$crate::Type::Fun($crate::ty!($a).into(), $crate::ty!($($b)+).into())
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
        $crate::rev_list!([$($rest),*] $first $(,$reversed)*)
    };
}

#[macro_export]
macro_rules! count {
    () => { 0 };
    ($x:ident $($xs:ident)*) => { 1 + $crate::count!($($xs)*)};
}
