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
	($([$($arg:pat_param),+] => $body:expr;)+) => {
		std::rc::Rc::new(|_args : &mut Vec<$crate::Term>|{
			#[allow(irrefutable_let_patterns)]
			#[allow(unused_variables)]
			match &_args[..] {
				$(
					rev_pat!([$($arg),+]) => {
						$(let $arg = _args.pop().unwrap() else {
							unimplemented!()
						};)*

						_args.push($body);

						true
					}
				)+
				_ => false
			}

		})
	}
}

#[macro_export]
macro_rules! rev_pat {
    ([] $($reversed:pat_param),*) => { 
        [.., $($reversed),*]
    };
    ([$first:pat_param $(, $rest:pat_param)*] $($reversed:pat_param),*) => { 
        rev_pat!([$($rest),*] $first $(,$reversed)*)
    };
}
