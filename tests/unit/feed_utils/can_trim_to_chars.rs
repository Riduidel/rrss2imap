use super::*;

#[test]
fn can_trim_empty_vec() {
	assert_eq!("",
		trim_to_chars("", vec![]));
}

#[test]
fn can_trim_simple_email() {
	assert_eq!(
		"a",
		trim_to_chars("a@b.c", vec!["@"])
	)
}