#[macro_export]
macro_rules! term {
	($x: ident) => {
		$crate::Term::Nam(stringify!($x), $x.clone().into())
	};
	($x: literal) => {
		if let Ok(n) = stringify!($x).parse::<i32>() {
			$crate::Term::Num(n)
		} else {
			$crate::Term::Var(&stringify!($x)[1..stringify!($x).len()-1])
		}
	};
	($x:literal -> $($r:tt)+) => {
		$crate::Term::Lam($x, term!($($r)+).into())
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
	($name:ident: $([$($arg:pat_param),+] => $body:expr;)+) => {{
		let func = std::rc::Rc::new(|_args : &mut Vec<$crate::Term>|{
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

		});

		$crate::Term::Lit(stringify!($name), func)
	}}
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
