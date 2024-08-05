fn main() {
	let fib: Vec<_> = (0..20).map(fib).collect();
}

fn fib(n: i32) -> i32 {
	if n < 1 {
		1
	} else {
		fib(n - 1) + fib(n - 2)
	}
}
