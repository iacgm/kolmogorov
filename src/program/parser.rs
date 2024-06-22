#[macro_export]
macro_rules! term {
	($x: ident) => {
		$crate::Term::Nam(stringify!($x), $x.clone().into())
	};
	([$x: ident]) => {
		$x.clone()
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
	($x:literal $($xs:literal)+ -> $($r:tt)+) => {
		$crate::Term::Lam($x, term!($($xs)* -> $($r)+).into())
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
	($name:ident: $([$($(!$(|$_:tt|)?)? $arg:ident),+] => $body:expr;)+) => {{
		use $crate::Term;
		use std::rc::Rc;

		let func = Rc::new(|_args : &mut Vec<Term>|{
			#[allow(irrefutable_let_patterns)]
			#[allow(unused_variables)]
			#[allow(unused_mut)]
			match &_args[..] {
				$(
					rev_list!([$($arg),+]) => {
						$(let mut $arg = _args.pop().unwrap() else {
							unimplemented!()
						};
						$(
							$arg.normalize();
							$arg.expand();
						$($_)?)?
						)*

						_args.push($body);

						true
					}
				)+
				_ => false
			}

		});

		Term::Lit(stringify!($name), func)
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
