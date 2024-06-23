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
	([$($arg:ident),+] => $body:expr) => {{
		use $crate::*;
		ContextEntry {
			active: true,
			n_args: count!($($arg)+),
			func: std::rc::Rc::new(move |_args| {
				let rev_list!([$($arg),+]) = &mut _args[..] else {
					unreachable!()
				};

				$body
			})
		}
	}}
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
