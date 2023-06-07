use super::*;

#[test]
fn can_sanitize_empty_list(){
	assert_eq!(vec!["a <a@example.com>".to_string()], 
		sanitize_message_authors(vec!["a".to_string()], "example.com".to_string()))
}
