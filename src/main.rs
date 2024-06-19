mod program;
pub use program::*;

fn main() {
    let mut i = term!((x -> x) 1);

    println!("{}", i);
    i.normalize(100);
    println!("{}", i);
}
