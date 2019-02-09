use requests;
use feed_rs::{parser,Entry};
use feed_rs::Feed as SourceFeed;
use chrono::NaiveDateTime;
use chrono::Utc;

use super::config::*;

use super::settings::*;

use super::entry::*;

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
        let url: String = consumed.pop().expect("You must at least define an url to add.");
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

    pub fn read(&self, settings:&Settings, config:&Config, email:&mut Imap) -> Feed{
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
                .for_each(|e| e.write_to_imap(&self, &feed, settings, config, email));
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
}
