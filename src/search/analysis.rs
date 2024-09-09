use super::*;

pub enum Semantics {
	Malformed,       // Reject Term entirely (i.e, unnecessarily complex)
	Unique,          // Allow, but did not construct canonical form
	Canonical(Term), // Group into equivalence class by canonical form
}
