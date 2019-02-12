use chrono::NaiveDateTime;

use super::settings::*;
use super::feed::*;
use super::config::*;

use std::io::Cursor;
use tera::Tera;
use tera::Context;

lazy_static! {
    pub static ref TERA:Tera = {
        let mut tera = compile_templates!("templates/*");
        tera.autoescape_on(vec![]);
        tera
    };
}

/// Provides last date for an element
pub trait Dated {
    fn last_date(&self)->NaiveDateTime;
}

pub trait Extractable<SourceFeed> : Dated {
    fn get_content(&self, settings:&Settings) -> String;
    fn get_title(&self, settings:&Settings) -> String;
    fn get_links(&self, settings:&Settings) -> Vec<String>;
    fn get_authors(&self, feed:&SourceFeed, settings:&Settings) -> Vec<String>;

    fn get_charset(&self, settings:&Settings) -> String {
        let text = self.get_content(settings);
        let mut text_cursor = Cursor::new(text.into_bytes());
        let detected_charsets = xhtmlchardet::detect(&mut text_cursor, None);
        match detected_charsets {
            Ok(charsets) => (&charsets[0]).to_owned(),
            Err(_) => "UTF-8".to_owned()
        }
    }

    /// Makes a valid HTML file out of the given Item.
    /// This method provides all the transformation that should happen
    fn extract_content(&self, feed:&SourceFeed, settings:&Settings) -> String {
        return TERA.render("message.html", &self.build_context(feed, settings)).unwrap();
    }

    fn write_to_imap(&self, feed:&Feed, source:&SourceFeed, settings:&Settings, config:&Config, email:&mut Imap) {
        let folder = feed.config.get_folder(config);
        let content = self.build_message(source, settings);
        match email.append(&folder, content) {
            Ok(_) => debug!("Successfully written {}", self.get_title(settings)),
            Err(e) => error!("{}\nUnable to select mailbox {}. Item titled {} won't be written", 
                    e, &folder, self.get_title(settings))
        }
    }
    fn build_context(&self, feed:&SourceFeed, settings:&Settings)->Context {
        let mut context = Context::new();
        context.insert("feed_entry", &self.get_content(settings));
        context.insert("links", &self.get_links(settings));
        context.insert("title", &self.get_title(settings));
        context.insert("from", &self.get_authors(feed, settings));
        context.insert("date", &self.last_date().format("%a, %d %b %Y %H:%M:%S -0000").to_string());
        return context;
    }

    fn build_message(&self, feed:&SourceFeed, settings:&Settings)->String {
        let mut context = self.build_context(feed, settings);
        context.insert("message_body", &self.extract_content(feed, settings));
        context.insert("charset", &self.get_charset(settings));
        return TERA.render("message.enveloppe", &context).unwrap();
    }
}

