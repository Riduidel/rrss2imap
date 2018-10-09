use requests;
use feed_rs::{parser,Entry};
use feed_rs::Feed as SourceFeed;
use chrono::NaiveDateTime;
use chrono::Utc;

use super::config::*;

use super::settings::*;

use tera::Tera;
use tera::Context;

lazy_static! {
    static ref TERA:Tera = {
        let mut tera = compile_templates!("templates/*");
        tera.autoescape_on(vec![]);
        tera
    };
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Feed {
    pub url: String,
    #[serde(skip_serializing_if = "Config::is_none", default = "Config::new")]
    pub config: Config,
    #[serde(default = "Feed::at_epoch")]
    pub last_updated: NaiveDateTime
}

impl Feed {
    /// Checks if feed has been read at least once.
    pub fn is_never_read(feed: &Feed)->bool {
        return feed.last_updated<=Feed::at_epoch();
    }
    /// Creates a new naivedatetime with a default value (which is, to my mind) a sensible default for computers
    pub fn at_epoch() -> NaiveDateTime {
        return NaiveDateTime::from_timestamp(0, 0);
    }

    // Convert the parameters vec into a valid feed (if possible)
    pub fn from(parameters: Vec<String>) -> Feed {
        let mut consumed = parameters.clone();
        let url: String = consumed.pop().unwrap();
        let mut email: Option<String> = None;
        let mut folder: Option<String> = None;
        // If there is a second parameter, it can be either email or folder
        if !consumed.is_empty() {
            let second = consumed.pop().unwrap();
            // If second parameters contains an @, I suppose it is an email address
            if second.contains("@") {
                debug!(
                    "Second add parameter {} is considered an email address",
                    second
                );
                email = Some(second)
            } else {
                warn!("Second add parameter {} is NOT considered an email address, but a folder. NO MORE ARGUMENTS WILL BE PROCESSED", second);
                folder = Some(second)
            }
        }
        // If there is a third parameter, it is the folder.
        // But if folder was already defined, there is an error !
        if !consumed.is_empty() && folder == None {
            folder = Some(consumed.pop().unwrap());
        }
        return Feed {
            url: url,
            config: Config {
                email: email,
                folder: folder,
            },
            last_updated: Feed::at_epoch()
        };
    }

    pub fn to_string(&self, config: &Config) -> String {
        return format!("{} {}", self.url, self.config.clone().to_string(config));
    }

    pub fn read(&self, settings:&Settings, config:&Config) -> Feed{
        info!("Reading feed from {}", self.url);
        let response = requests::get(&self.url).unwrap();
        let feed_content_as_text = response.text().unwrap();
        // Now parse it as XML, because it is XML
        let feed = parser::parse(&mut feed_content_as_text.as_bytes()).unwrap();
        let feed_date = feed.last_updated.unwrap_or(Utc::now().naive_utc());
        info!("Feed date is {} while previous read date is {}", feed_date, self.last_updated);
        if feed_date>=self.last_updated {
            info!("There should be new entries, parsing HTML content");
            feed.entries.iter()
                .filter(|e| e.last_date()>=self.last_updated)
                .map(|e| self.process_entry(&feed, e, settings))
                .for_each(|e| self.write_to_imap(&feed, e, settings, config));
            return Feed {
                url: self.url.clone(),
                config: self.config.clone(),
                last_updated: if settings.do_not_save { self.last_updated.clone() } else { feed_date }
            };
        } else {
            return Feed {
                url: self.url.clone(),
                config: self.config.clone(),
                last_updated: self.last_updated.clone()
            };
        }
    }

    /// Processing an entry will generate, from the entry, the "correct" HTML fragment :
    /// a table able to be rendered in mail client and containing the needed informations
    /// returns a transformed entry which can be directly serialized into an IMAP message
    fn process_entry(&self, feed:&SourceFeed, entry:&Entry, settings:&Settings) -> Entry{
        info!("Processing entry {} written at {}", entry.id, entry.last_date());
        let mut returned = Entry::new();
        returned.id = entry.clone().id;
        returned.title = Some(entry.clone().title.unwrap_or(feed.clone().title.unwrap()));
        returned.published = entry.published;
        returned.updated = Some(entry.last_date());
        returned.author = entry.clone().author;
        returned.content = Some(entry.extract_content(settings));
        return returned;
    }

    fn write_to_imap(&self, feed:&SourceFeed, entry:Entry, settings:&Settings, config:&Config) {
        info!("Full content is\n{:?}\n\n\n\n\n", entry);
    }
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
    fn get_content(&self) -> String;
    fn get_title(&self) -> String;
    fn get_link(&self) -> String;
    fn extract_content(&self, settings:&Settings) -> String;
}

impl Extractable for Entry {
    fn get_content(&self) -> String {
        return self.clone().content.unwrap_or(self.clone().summary.unwrap());
    }
    fn get_title(&self) -> String {
        return self.clone().title.unwrap();
    }
    fn get_link(&self) -> String {
        return self.clone().id;
    }
    fn extract_content(&self, settings:&Settings) -> String {
        let mut context = Context::new();
        context.insert("content", &self.get_content());
        context.insert("link", &self.get_link());
        context.insert("title", &self.get_title());
        return TERA.render("message.html", &context).unwrap();
    }

}