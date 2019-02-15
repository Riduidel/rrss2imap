use chrono::{NaiveDateTime, DateTime, Utc};

use super::config::*;

use super::settings::*;
use super::extractable::*;
use super::syndication;
use rss::Channel;
use atom_syndication::Feed as AtomFeed;

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
        feed.last_updated<=Feed::at_epoch()
    }
    /// Creates a new naivedatetime with a default value (which is, to my mind) a sensible default for computers
    pub fn at_epoch() -> NaiveDateTime {
        NaiveDateTime::from_timestamp(0, 0)
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
            if second.contains('@') {
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
        Feed {
            url,
            config: Config {
                email,
                folder,
            },
            last_updated: Feed::at_epoch()
        }
    }

    pub fn to_string(&self, config: &Config) -> String {
        return format!("{} {}", self.url, self.config.clone().to_string(config));
    }

    pub fn read(&self, settings:&Settings, config:&Config, email:&mut Imap) -> Feed{
        info!("Reading feed from {}", self.url);
        let response = requests::get(&self.url).unwrap();
        if response.ok() {
            let text = response.text().unwrap();
            let parsed = text.parse::<syndication::Feed>().unwrap();
            match parsed {
                syndication::Feed::Atom(atom_feed) => self.read_atom(atom_feed, settings, config, email),
                syndication::Feed::RSS(rss_feed) => self.read_rss(rss_feed, settings, config, email)
            }
        } else {
            error!("HTTP code is {} when trying to get feed {}", response.status_code(), &self.url);
            self.clone()
        }
    }

    fn read_atom(&self, feed:AtomFeed, settings:&Settings, config:&Config, email:&mut Imap) -> Feed{
        info!("reading ATOM feed {}", &self.url);
        let feed_date_text = feed.updated();
        let feed_date = feed_date_text.parse::<DateTime<Utc>>().unwrap().naive_utc();
        info!("Feed date is {} while previous read date is {}", feed_date, self.last_updated);
        if feed_date>=self.last_updated {
            info!("There should be new entries, parsing HTML content");
            feed.entries().iter()
                .filter(|e| e.last_date()>=self.last_updated)
                .for_each(|e| e.write_to_imap(&self, &feed, settings, config, email));
            return Feed {
                url: self.url.clone(),
                config: self.config.clone(),
                last_updated: if settings.do_not_save { self.last_updated } else { feed_date }
            };
        }        
        self.clone()
    }

    fn read_rss(&self, feed:Channel, settings:&Settings, config:&Config, email:&mut Imap) -> Feed{
        info!("reading RSS feed {}", &self.url);
        let feed_date_text = feed.pub_date().unwrap_or_else(|| feed.last_build_date().unwrap());
        let feed_date = DateTime::parse_from_rfc2822(&feed_date_text).unwrap().naive_utc();
        info!("Feed date is {} while previous read date is {}", feed_date, self.last_updated);
        if feed_date>=self.last_updated {
            info!("There should be new entries, parsing HTML content");
            feed.items().iter()
                .filter(|e| e.last_date()>=self.last_updated)
                .for_each(|e| e.write_to_imap(&self, &feed, settings, config, email));
            return Feed {
                url: self.url.clone(),
                config: self.config.clone(),
                last_updated: if settings.do_not_save { self.last_updated } else { feed_date }
            };
        }        
        self.clone()
    }
}
