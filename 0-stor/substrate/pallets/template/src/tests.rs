use crate::{mock::*};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		let namespace = Vec::from("test".as_bytes());
		let key = Vec::from("a".as_bytes());

		let hash = super::contruct_hashkey(&namespace, &key);

		assert_eq!(hash.len(), 17);

		let hash_hash = hex::encode(hash);
		assert_eq!(hash_hash, "c66a28419644149496ce3354be3a035d61")
	});
}
