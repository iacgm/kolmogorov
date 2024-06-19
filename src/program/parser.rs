#[macro_export]
macro_rules! term {
	($x: ident) => {
		$crate::Term::Var(stringify!($x))	
	};
	($x: literal) => {
		$crate::Term::Num($x)
	};
	($x:ident -> $r:tt) => {
		$crate::Term::Lam(stringify!($x), Box::new(term!($r)))
	};
	(($($r:tt)*)) => {
		term!($($r)*)
	};
	($($r:tt)*) => {
		$crate::Term::App(vec![$(term!($r)),*])
	};
}
