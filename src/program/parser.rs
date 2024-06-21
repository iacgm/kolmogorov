#[macro_export]
macro_rules! term {
	(@$x: ident) => {
		$crate::Term::Lit(stringify!($x), $x)
	};
	($x: ident) => {
		$crate::Term::Var(stringify!($x))	
	};
	($x: literal) => {
		$crate::Term::Num($x)
	};
	($x:ident -> $($r:tt)+) => {
		$crate::Term::Lam(stringify!($x), term!($($r)+).into())
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
	([$($arg:pat_param),+] => $body:expr) => {
		std::rc::Rc::new(|_args : &mut Vec<$crate::Term>|{
			if (_args.len() < count!($($arg),+)){
				return false
			}

			$(let $arg = _args.pop().unwrap() else {
				unimplemented!()
			};)*

			_args.push($body);

			true
		})
	}
}

#[macro_export]
macro_rules! count {
    () => (0);
	($x:pat_param) => (1);
    ($x:pat_param, $($xs:pat_param),*) => (1 + count!($($xs),*));
}
