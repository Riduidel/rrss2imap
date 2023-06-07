extern crate spectral;
use spectral::prelude::*;

use std::env;
use crate::{settings::Email, config::Config};
use std::fs;
use super::*;

#[test]
fn can_read_store_from_existing_file() {
	let mut config_file = env::current_dir().unwrap();
	config_file.push("tests");
	config_file.push("unit");
	config_file.push("store");
	config_file.push("simple_config_store.json");
	assert_that!(config_file)
		.is_a_file();
	let store_result = Store::load(&config_file);
	assert_that!(store_result)
		.is_ok();
	let store = store_result.unwrap();
	assert_that!(store.settings.email)
	.is_equal_to(Email {
		server: "imap_server".to_string(),
		user: "username".to_string(),
		password: "password".to_string(),
		secure: crate::settings::Secure::Yes(993),
		retry_max_count: 3,
		retry_delay: 1
	});
	assert_that!(store.settings.config)
	.is_equal_to(Config {
		email: Some("Sender <username@imap_server.com>".to_string()),
		folder: Some("default_folder".to_string()),
		from: None,
		inline_image_as_data: true
	});
	assert_that!(store.feeds)
		.has_length(1);
}


#[test]
fn can_read_store_from_non_existing_file() {
	let mut config_file = env::current_dir().unwrap();
	config_file.push("tests");
	config_file.push("unit");
	config_file.push("store");
	config_file.push("can_read_store_from_non_existing_file.json");
	assert_that!(config_file)
		.does_not_exist();
	let store_result = Store::load(&config_file);
	assert_that!(store_result)
		.is_ok();
	let store = store_result.unwrap();
	assert_that!(store.settings.email)
	.is_equal_to(Email {
		server: "Set your email server address here".to_string(),
		user: "Set your imap server user name (it may be your email address or not)".to_string(),
		password: "Set your imap server password (yup, in clear, this is very bad)".to_string(),
		secure: crate::settings::Secure::Yes(993),
		retry_max_count: 3,
		retry_delay: 1
	});
	assert_that!(store.settings.config)
	.is_equal_to(Config {
		email: None,
		folder: None,
		from: None,
		inline_image_as_data: false
	});
	assert_that!(store.feeds)
	.is_equal_to(vec![])
}

#[test]
fn bugfix_82_export_is_broken() {
	// Given
	let mut store_path = env::current_dir().unwrap();
	store_path.push("tests");
	store_path.push("unit");
	store_path.push("store");
	store_path.push("bugfix_82_export_is_broken.json");
	let mut store = Store {
		settings: Settings { 
			do_not_save: false, 
			email: Email {
				server: "imap_server".to_string(),
				user: "username".to_string(),
				password: "password".to_string(),
				secure: crate::settings::Secure::Yes(993),
				retry_max_count: 3,
				retry_delay: 1
			}, 
			config: Config {
				email: Some("Sender <username@imap_server.com>".to_string()),
				folder: Some("default_folder".to_string()),
				from: None,
				inline_image_as_data: true
			}
		},
		feeds: vec![],
		dirty: true,
		path: store_path
	};
	let mut export_path = env::current_dir().unwrap();
	export_path.push("tests");
	export_path.push("unit");
	export_path.push("store");
	export_path.push("export.opml");
	// Finally, add one feed to the store
	store.add_feed(Feed::from_vec(vec!["https://xkcd.com/rss.xml".to_string()]));
	// When
	store.export(Some(export_path.clone()));
	// Then
	assert_that!(export_path)
		.is_a_file();
	// Read file content to a string
	let opml_content = fs::read_to_string(export_path).unwrap();
	assert_that!(opml_content)
		.contains("https://xkcd.com/rss.xml")
}