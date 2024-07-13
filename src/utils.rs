pub fn probability(p: f32) -> bool {
	random() < p
}

pub fn random() -> f32 {
	rand::random::<f32>()
}
