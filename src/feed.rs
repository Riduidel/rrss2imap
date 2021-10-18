use chrono::{NaiveDateTime};

use super::config::*;

use super::feed_reader::*;
use super::settings::*;
use super::syndication;
use reqwest::blocking::Client;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Feed {
    /// Contains url of feed
    pub url: String,
    /// Contains specific configuration for field
    #[serde(skip_serializing_if = "Config::is_none", default = "Config::new")]
    pub config: Config,
    /// Last time the feed was read
    #[serde(default = "Feed::at_epoch")]
    pub last_updated: NaiveDateTime,
    /// Last message stored in IMAP, allows to correctly process feeds even when no date is provided
    /// which, mind you, is totally possible according to RSS specification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message: Option<String>
}

impl Feed {
    /// Creates a new naivedatetime with a default value (which is, to my mind) a sensible default for computers
    pub fn at_epoch() -> NaiveDateTime {
        NaiveDateTime::from_timestamp(0, 0)
    }

    // Convert the parameters vec into a valid feed (if possible)
    pub fn from_vec(parameters: Vec<String>) -> Feed {
        let mut consumed = parameters;
        let url: String = consumed
            .pop()
            .expect("You must at least define an url to add.");
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
                from: None,
                inline_image_as_data: false,
            },
            last_updated: Feed::at_epoch(),
            last_message: None
        }
    }

    pub fn from_all(url:Option<String>, email:Option<String>, destination:Option<String>, inline:bool) -> Feed {
        Feed {
            url: url.unwrap(),
            config: Config {
                email,
                folder: destination,
                from: None,
                inline_image_as_data: inline,
            },
            last_updated: Feed::at_epoch(),
            last_message: None
        }
    }

    pub fn to_string(&self, config: &Config) -> String {
        return format!("{} {}", self.url, self.config.clone().to_string(config));
    }

    pub async fn read(&self, index:usize, count:&usize, client:&Client, settings: &Settings) -> Feed {
        info!("Reading feed {}/{} from {}", index+1, count, self.url);
        match client.get(&self.url).send() {
            Ok(response) => match response.text() {
                Ok(text) => match text.parse::<syndication::Feed>() {
                    Ok(parsed) => {
                        return match parsed {
                            syndication::Feed::Atom(atom_feed) => {
                                AtomReader {}.read(self, &atom_feed, &settings)
                            }
                            syndication::Feed::RSS(rss_feed) => {
                                RssReader {}.read(self, &rss_feed, &settings)
                            }
                        }
                    }
                    Err(e) => error!("Content ar {} is neither Atom, nor RSS {}.\nTODO check real content type to help user.", &self.url, e),
                },
                Err(e) => error!("There is no text at {} due to error {}", &self.url, e),
            },
            Err(e) => error!("Unable to get {} due to {}.\nTODO Add better http response analysis !", &self.url, e),
        }
        self.clone()
    }

}
