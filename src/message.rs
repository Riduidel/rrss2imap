use chrono::NaiveDateTime;

use super::feed::Feed;
use super::settings::*;
use super::image_to_data;
use std::io::Cursor;
use tera::Context;
use tera::Tera;

use kuchiki::traits::*;

lazy_static! {
    pub static ref TERA: Tera = {
        let mut tera = compile_templates!("templates/*");
        tera.autoescape_on(vec![]);
        tera
    };
}


///
/// Structure for storing message data prior to having these messages written to IMAP.
/// This structure serves as a common interface for Item/Entry
pub struct Message {
    pub authors:Vec<String>,
    pub content:String,
    pub id:String,
    pub last_date:NaiveDateTime,
    pub links:Vec<String>,
    pub title:String,
}

impl Message {
    pub fn write_to_imap(&self, feed:&Feed, settings:&Settings, email:&mut Imap) {
        let folder = feed.config.get_folder(&settings.config);
        let content = self.build_message(feed, settings);
        match email.append(&folder, content) {
            Ok(_) => debug!("Successfully written {}", self.title),
            Err(e) => error!(
                "{}\nUnable to select mailbox {}. Item titled {} won't be written",
                e,
                &folder,
                self.title
            ),
        }
    }

    fn get_charset(&self, text: &String, _settings: &Settings) -> String {
        let mut text_cursor = Cursor::new(text.clone().into_bytes());
        let detected_charsets = xhtmlchardet::detect(&mut text_cursor, None);
        match detected_charsets {
            Ok(charsets) => (&charsets[0]).to_owned(),
            Err(_) => "UTF-8".to_owned(),
        }
    }

    fn build_message(&self, feed: &Feed, settings: &Settings) -> String {
        let mut context = self.build_context(feed, settings);
        let content = self.extract_content(feed, settings);
        context.insert("message_body", &base64::encode(&content));
        context.insert("charset", &self.get_charset(&content, settings));
        TERA.render("message.enveloppe", &context).unwrap()
    }


    /// Makes a valid HTML file out of the given Item.
    /// This method provides all the transformation that should happen
    fn extract_content(
        &self,
        feed: &Feed,
        settings: &Settings,
    ) -> String {
        TERA.render(
            "message.html",
            &self.build_context(feed, settings),
        )
        .unwrap()
    }

    ///
    /// Process the feed effective content.
    /// This should allow
    /// * image transformation into base64 when needed
    ///
    fn get_processed_content(&self, feed: &Feed, settings: &Settings) -> String {
        let mut document = kuchiki::parse_html().one(self.content.clone());
        if feed.config.inline_image_as_data || settings.config.inline_image_as_data {
            // So, take content, pass it through html5ever (thanks to select, and transform each image !)
            debug!(
                "We should inline image as base64 data for {}",
                self.id
            );
            document = image_to_data::transform(document, feed, settings);
        }
        let mut bytes = vec![];
        document.serialize(&mut bytes).expect(&format!(
            "Unable to read entry {:?} of feed {}\nConsider sending a bug report !",
            self.links,
            feed.url
        ));
        return String::from_utf8(bytes).unwrap();
    }

    fn build_context(&self, feed: &Feed, settings: &Settings) -> Context {
        let mut context = Context::new();
        context.insert("feed_entry", &self.get_processed_content(feed, settings));
        context.insert("links", &self.links);
        context.insert("id", &self.id);
        context.insert("title", &self.title);
        context.insert("from", &self.authors);
        context.insert("to", &feed.config.get_email(&settings.config));
        context.insert(
            "date",
            &self
                .last_date
                .format("%a, %d %b %Y %H:%M:%S -0000")
                .to_string(),
        );
        context
    }
}