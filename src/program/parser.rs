#[macro_export]
macro_rules! term {
	($x: ident) => {
		$crate::Term::Var(stringify!($x))
	};
	([$x: ident]) => {
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
macro_rules! literal {
	($($ty:tt)=>* [$($arg:ident),+] => $body:expr) => {{
		use $crate::*;
		let n_args = count!($($arg)+);

		ContextEntry {
			active: true,
			n_args,
			ty: ty!($($ty)=>+),
			func: std::rc::Rc::new(move |_args| {
				let i = 1;
				let rev_list!([$($arg),+]) = &mut _args[..] else {
					unreachable!()
				};

				$(
					let $arg = std::mem::replace($arg, Num(0));
				)+ 

				$body
			})
		}
	}}
}

#[macro_export]
macro_rules! ty {
	(N) => {
		$crate::Type::Nat
	};
	($x:ident) => {
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
        [.., $($reversed),*]
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
