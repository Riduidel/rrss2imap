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

/// Makes sure we can parse the feed given in https://validator.w3.org/feed/docs/atom.html
#[test]
fn can_read_an_atom_feed() {
	let feed = Feed::from_vec(vec!["a@b.c".to_string()]);
	let messages = feed.read_response_text(include_str!("example.atom").to_string());
	assert_that!(messages)
		.has_length(1)
		;
	let first = &messages[0];
	assert_that!(first.content).is_equal_to("Some text.".to_string());

}


/// Makes sure we can parse the feed given in https://en.wikipedia.org/wiki/RSS?useskin=vector#Example
#[test]
fn can_read_a_rss_feed() {
	let feed = Feed::from_vec(vec!["a@b.c".to_string()]);
	let messages = feed.read_response_text(include_str!("example.rss").to_string());
	assert_that!(messages)
		.has_length(1)
		;
	let first = &messages[0];
	assert_that!(first.content).is_equal_to("Here is some text containing an interesting description.".to_string());

}