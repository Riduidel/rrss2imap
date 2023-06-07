extern crate spectral;
use spectral::prelude::*;
use super::*;

#[test]
fn can_build_feed_from_just_a_url() {
	assert_that!(
		Feed::from_vec(vec!["example.com".to_string()])
	).is_equal_to(
		Feed {
			url: "example.com".to_string(),
			config: Config {
				email: None,
				folder: None,
				from: None,
				inline_image_as_data: false
			},
			last_updated: Feed::at_epoch(),
			last_message: None
		})
}

#[test]
fn can_build_feed_from_url_and_email() {
	assert_that!(
		Feed::from_vec(vec!["a@example.com".to_string(),"example.com".to_string()])
	).is_equal_to(
		Feed {
			url: "example.com".to_string(),
			config: Config {
				email: Some("a@example.com".to_string()),
				folder: None,
				from: None,
				inline_image_as_data: false
			},
			last_updated: Feed::at_epoch(),
			last_message: None
		})
}

#[test]
fn can_build_feed_from_url_and_folder() {
	assert_that!(
		Feed::from_vec(vec!["folder".to_string(), "example.com".to_string()])
	).is_equal_to(
		Feed {
			url: "example.com".to_string(),
			config: Config {
				email: None,
				folder: Some("folder".to_string()),
				from: None,
				inline_image_as_data: false
			},
			last_updated: Feed::at_epoch(),
			last_message: None
		})
}

#[test]
fn can_build_feed_from_url_email_and_folder() {
	assert_that!(
		Feed::from_vec(vec!["a@b.c".to_string(), "folder".to_string(), "example.com".to_string()])
	).is_equal_to(
		Feed {
			url: "example.com".to_string(),
			config: Config {
				email: None,
				folder: Some("folder".to_string()),
				from: None,
				inline_image_as_data: false
			},
			last_updated: Feed::at_epoch(),
			last_message: None
		})
}
