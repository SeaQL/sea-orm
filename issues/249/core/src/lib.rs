pub use sea_orm::entity::*;

pub fn clone_a_model<M>(model: &M) -> M
where
	M: ModelTrait {
	model.clone()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test() {
		println!("OK");
	}
}