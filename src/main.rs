mod program;
pub use program::*;

fn main() {

    use Term::*;
    let sum = literal!(
        [Num(x), Num(y)] => Num(x+y);
        [x, y] => Num(-1);
    );
    
    let mut code = term!((@sum) 1 (x -> x));

    println!("{}", code);
    code.bounded_normalize(100);
    println!("{}", code);
}
