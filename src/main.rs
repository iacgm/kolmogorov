mod program;
pub use program::*;

fn main() {
    use Term::*;
    let sum = literal!([Num(x), Num(y)] => {
        Num(x+y)
    });
    
    let mut code = term!((@sum) 1 2);

    println!("{}", code);
    code.bounded_normalize(100);
    println!("{}", code);
}
