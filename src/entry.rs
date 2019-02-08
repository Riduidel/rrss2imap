use chrono::NaiveDateTime;

use super::settings::*;
use super::feed::*;
use super::config::*;

use feed_rs::Entry;
use feed_rs::Feed as SourceFeed;

use tera::Tera;
use tera::Context;

use kuchiki::*;
use kuchiki::traits::*;

use base64::encode;

lazy_static! {
    static ref TERA:Tera = {
        let mut tera = compile_templates!("templates/*");
        tera.autoescape_on(vec![]);
        tera
    };
}

/// Provides last date for an element
pub trait Dated {
    fn last_date(&self)->NaiveDateTime;
}
impl Dated for Entry {
    fn last_date(&self)->NaiveDateTime {
        return self.updated.unwrap_or(self.published);
    }

}
pub trait Extractable {
    fn get_content(&self, settings:&Settings) -> String;
    fn get_title(&self, settings:&Settings) -> String;
    fn get_link(&self, settings:&Settings) -> String;
    fn get_author(&self, feed:&SourceFeed, settings:&Settings) -> String;
    /// Makes a valid HTML file out of the given entry.
    /// This method provides all the transformation that should happen
    fn extract_content(&self, feed:&SourceFeed, settings:&Settings) -> String;

    fn write_to_imap(&self, feed:&Feed, source:&SourceFeed, settings:&Settings, config:&Config, email:&mut Imap);

}

impl Extractable for Entry {
    fn get_content(&self, settings:&Settings) -> String {
        let text = self.clone().content.unwrap_or(self.clone().summary.unwrap_or("".to_owned()));
        // First step is to fix HTML, so load it using html5ever 
        // (because there is no better html parser than a real browser one)
        // TODO implement image inlining
        return text;
    }
    fn get_title(&self, _settings:&Settings) -> String {
        return self.clone().title.unwrap();
    }
    fn get_link(&self, _settings:&Settings) -> String {
        return self.clone().id;
    }
    fn get_author(&self, feed:&SourceFeed, _settings:&Settings) -> String {
        return self.clone().author.unwrap_or(feed.clone().title.unwrap_or("no author found".to_owned()));
    }
    fn extract_content(&self, feed:&SourceFeed, settings:&Settings) -> String {
        info!("calling extract_content for {}", self.get_link(settings));
        let mut context = Context::new();
        context.insert("feed_entry", &self.get_content(settings));
        context.insert("link", &self.get_link(settings));
        context.insert("title", &self.get_title(settings));
        context.insert("from", &self.get_author(feed, settings));
        context.insert("date", &self.last_date().format("%a, %d %b %Y %H:%M:%S -0000").to_string());
        let body = TERA.render("message.html", &context).unwrap();
        context.insert("message_body", &body);
        return TERA.render("message.enveloppe", &context).unwrap();
    }

    fn write_to_imap(&self, feed:&Feed, source:&SourceFeed, settings:&Settings, config:&Config, email:&mut Imap) {
        let folder = feed.config.get_folder(config);
        match email.append(&folder, self.extract_content(source, settings)) {
            Ok(_) => debug!("Successfully written {}", self.get_title(settings)),
            Err(e) => error!("{}\nUnable to select mailbox {}. Entry titled {} won't be written", 
                    e, &folder, self.get_title(settings))
        }
    }
}
