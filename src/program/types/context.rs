use super::*;

use std::collections::HashMap;

pub struct TypingContext {
	free: HashSet<Identifier>,
	bound: HashMap<Identifier, MonoType>,
}

impl TypingContext {
	
}
