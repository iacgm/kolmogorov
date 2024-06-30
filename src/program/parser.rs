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
		|$($arg:ident),+| => $body:expr
	) => {{
		use $crate::*;

		let ty = ty!($(forall $($a)+ ::)? $($ty)=>+);

		let n_args = count!($($arg)+);

		$($(
			let $captured = $captured.clone();
		)+)?

		ContextEntry {
			active: true,
			n_args,
			ty,
			func: std::rc::Rc::new(move |_args| {
				let rev_list!([$($arg),+]) = &mut _args[..] else {
					unreachable!()
				};

				$(
					let $arg = std::mem::replace($arg, $crate::Term::Num(0));
				)+ 

				$body
			})
		}
	}}
}

#[macro_export]
macro_rules! ty {
	(N) => {
		$crate::Type::Int
	};
	(Int) => {
		$crate::Type::Int
	};
	(Bool) => {
		ty!(forall _t :: _t => _t => _t)
	};
	($x:ident) => {
		$crate::Type::Name(stringify!($x))
	};
	(forall $a:ident :: $($b:tt)*) => {
		$crate::Type::Poly(stringify!($a), ty!($($b)*).into())
	};
	(forall $a:ident $($as:ident)+ :: $($b:tt)*) => {
		$crate::Type::Poly(stringify!($a), ty!(forall $($as)+ : $($b)*).into())
	};
	([$a:tt]) => {
		ty!(forall _t :: _t => ($a => _t => _t) => _t)
	};
	($a:tt => $($b:tt)+) => {
		$crate::Type::Func(ty!($a).into(), ty!($($b)+).into())
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
