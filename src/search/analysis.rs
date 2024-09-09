use super::*;

pub enum Analysis {
	Malformed,       // Reject Term entirely (i.e, unnecessarily complex)
	Unique,          // Allow, but did not construct canonical form
	Canonical(Term), // Group into equivalence class by canonical form
}

impl std::fmt::Display for Analysis {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use Analysis::*;
		match self {
			Malformed => write!(f, "Malformed"),
			Unique => write!(f, "Unique"),
			Canonical(term) => write!(f, "Canonical({})", term),
		}
	}
}
