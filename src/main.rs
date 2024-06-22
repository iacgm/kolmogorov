mod program;
pub use program::*;

fn main() {

    use Term::*;
    let sum = literal!(sum:
        [Num(x), Num(y)] => Num(x+y);
        [x, y] => Num(-1);
    );
    
    let cons = term!("h" -> "t" -> "s" -> "s" "h" "t");
    let t = term!("x" -> "y" -> "x");
    let f = term!("x" -> "y" -> "y");
    let nil = term!("nil");

    let isnil = literal!(isnil:
        [Var("nil")] => term!("x" -> "y" -> "x");
        [_] => term!("x" -> "y" -> "y");
    );

    let head = term!("l" -> "l" t);
    let tail = term!("l" -> "l" f);

    let make_list = |elems: &[Term]| {
        let mut out = term!(nil);
        for elem in elems.iter().rev() {
            out = term!(cons [elem] [out]);
        }
        out
    };
    
    let list = make_list(&[term!(1),term!(2),term!(3)]);
    println!("list: {}", list);

    exec(&list);
 	exec(&term!(head list));
 	exec(&term!(head (tail list)));
	exec(&term!(head (tail (tail list))));
	exec(&term!(head (tail (tail (tail list)))));

}

fn exec(term: &Term) {
    let mut term = term.clone();
	println!("{}:", term);	
	term.normalize();
    println!("\t{}", term);
}
