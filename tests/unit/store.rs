extern crate spectral;
use spectral::prelude::*;
use chrono::NaiveDateTime;
use std::env;
use crate::{settings::Email, config::Config};

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
	config_file.push("this_file_doesnt_exist.json");
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