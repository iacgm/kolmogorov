use super::*;

struct Scoped {
	lang: Box<dyn Language>,
	ctxt: Context,
}

impl Scoped {
	pub fn new(lang: Box<dyn Language>, scope: &Context) -> Self {
		let mut ctxt = lang.context();

		let defs: Vec<_> = scope.iter().map(|(&i, b)| (i, b.clone())).collect();
		ctxt.insert(&defs);

		Self { lang, ctxt }
	}
}

impl Language for Scoped {
	fn context(&self) -> Context {
		self.ctxt.clone()
	}
}
