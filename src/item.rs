use chrono::NaiveDateTime;

use super::settings::*;
use super::feed::*;
use super::config::*;

use rss::Item;
use rss::Channel as SourceFeed;

use chrono::DateTime;

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
impl Dated for Item {
    fn last_date(&self)->NaiveDateTime {
        return DateTime::parse_from_rfc2822(self.pub_date().unwrap()).unwrap().naive_utc();
    }

}
pub trait Extractable {
    fn get_content(&self, settings:&Settings) -> String;
    fn get_title(&self, settings:&Settings) -> String;
    fn get_link(&self, settings:&Settings) -> String;
    fn get_author(&self, feed:&SourceFeed, settings:&Settings) -> String;
    /// Makes a valid HTML file out of the given Item.
    /// This method provides all the transformation that should happen
    fn extract_content(&self, feed:&SourceFeed, settings:&Settings) -> String;

    fn write_to_imap(&self, feed:&Feed, source:&SourceFeed, settings:&Settings, config:&Config, email:&mut Imap);

}

impl Extractable for Item {
    fn get_content(&self, settings:&Settings) -> String {
        let text = self.content().unwrap_or(self.description().unwrap_or(""));
        // First step is to fix HTML, so load it using html5ever 
        // (because there is no better html parser than a real browser one)
        // TODO implement image inlining
        return text.to_owned();
    }
    fn get_title(&self, _settings:&Settings) -> String {
        return self.clone().title().unwrap().to_owned();
    }
    fn get_link(&self, _settings:&Settings) -> String {
        return self.clone().link().unwrap().to_owned();
    }
    fn get_author(&self, feed:&SourceFeed, _settings:&Settings) -> String {
        return self.clone().author().unwrap_or(feed.clone().title()).to_owned();
    }
    fn extract_content(&self, feed:&SourceFeed, settings:&Settings) -> String {
        debug!("calling extract_content for {}", self.get_link(settings));
        return TERA.render("message.html", &build_context(self, feed, settings)).unwrap();
    }

    fn write_to_imap(&self, feed:&Feed, source:&SourceFeed, settings:&Settings, config:&Config, email:&mut Imap) {
        let folder = feed.config.get_folder(config);
        let content = build_message(self, source, settings);
        match email.append(&folder, content) {
            Ok(_) => debug!("Successfully written {}", self.get_title(settings)),
            Err(e) => error!("{}\nUnable to select mailbox {}. Item titled {} won't be written", 
                    e, &folder, self.get_title(settings))
        }
    }
}

fn build_context(entry:&Item, feed:&SourceFeed, settings:&Settings)->Context {
    let mut context = Context::new();
    context.insert("feed_entry", &entry.get_content(settings));
    context.insert("link", &entry.get_link(settings));
    context.insert("title", &entry.get_title(settings));
    context.insert("from", &entry.get_author(feed, settings));
    context.insert("date", &entry.last_date().format("%a, %d %b %Y %H:%M:%S -0000").to_string());
    return context;
}

fn build_message(entry:&Item, feed:&SourceFeed, settings:&Settings)->String {
    debug!("calling build_message for {}", entry.get_link(settings));
    let mut context = build_context(entry, feed, settings);
    context.insert("message_body", &entry.extract_content(feed, settings));
    return TERA.render("message.enveloppe", &context).unwrap();
}