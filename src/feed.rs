use chrono::{NaiveDateTime};
use tests_bin::unit_tests;

use super::config::*;

use super::feed_reader::*;
use super::settings::*;
use super::syndication;
use super::message::*;

#[unit_tests("feed.rs")]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
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
        NaiveDateTime::from_timestamp_opt(0, 0).unwrap()
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
        if !consumed.is_empty() && folder.is_none() {
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
        format!("{} {}", self.url, self.config.clone().to_string(config))
    }

    /**
     * Read the feed and produce the list of messages to write later
     */
    pub fn read(&self, index:usize, count:&usize) -> Vec<Message> {
        info!("Reading feed {}/{} from {}", index+1, count, self.url);
        match ureq::get(&self.url).call() {
            Ok(response) => match response.into_string() {
                Ok(text) => match text.parse::<syndication::Feed>() {
                    Ok(parsed) => {
                        return match parsed {
                            syndication::Feed::Atom(atom_feed) => {
                                AtomReader {}.read(self, &atom_feed)
                            }
                            syndication::Feed::RSS(rss_feed) => {
                                RssReader {}.read(self, &rss_feed)
                            }
                        }
                    }
                    Err(e) => error!("Content ar {} is neither Atom, nor RSS {}.\nTODO check real content type to help user.", &self.url, e),
                },
                Err(e) => error!("There is no text at {} due to error {}", &self.url, e),
            },
            Err(e) => error!("Unable to get {} due to {}.\nTODO Add better http response analysis !", &self.url, e),
        }
        vec![]
    }

    pub fn process_message(&self, settings:&Settings, message:&Message)->Message {
        Message {
            authors: message.authors.clone(),
            content: Message::get_processed_content(&message.content, self, settings).unwrap(),
            id: message.id.clone(),
            last_date: message.last_date,
            links: message.links.clone(),
            title: message.title.clone(),
        }
    }

    /// Find in the given input feed the new messages
    /// A message is considered new if it has a date which is nearer than feed last processed date
    /// or (because RSS and Atom feeds may not have dates) if its id is not yet the id of the last
    /// processed feed
    pub fn find_new_messages(&self, sorted_messages:&[Message])->(usize, usize, bool) {
        let head:usize = 0;
        let mut tail:usize = 0;
        let mut found = false;
        // Now do the filter
        // This part is not so easy.
        // we will first iterate over the various items and for each, check that
        // 1 - the message id is not the last read message one
        // 2 - if messages have dates, the message date is more recent than the last one
        for (position, message) in sorted_messages.iter().enumerate() {
            if !found {
                match &self.last_message {
                    Some(id) => if id==&message.id {
                        tail = position; 
                        found = true;
                        break;
                    },
                    None => {}
                };
                if message.last_date<self.last_updated {
                    tail = position; 
                    found = true;
                }
            }
        }
        (head, tail, found)
    }

    pub fn write_new_messages(&self, settings:&Settings, extracted:Vec<Message>)->Feed {
        let sorted_messages = extracted;
        let (head, tail, found) = self.find_new_messages(sorted_messages.as_slice());
        let filtered_messages:&[Message] = if found {
            &sorted_messages[head..tail]
        } else {
            sorted_messages.as_slice()
        };

        // And write the messages into IMAP and the feed into JSON
        let written_messages:Vec<Message> = filtered_messages.iter()
            .map(|message| self.process_message(settings, message))
            .inspect(|e| if !settings.do_not_save { e.write_to_imap(self, settings) } )
            .collect();
        let mut last_message:Option<&Message> = written_messages.iter()
            // ok, there is a small problem here: if at least two elements have the same value - which is the case when feed
            // elements have no dates - the LAST one is used (which is **not** what we want)
            // see https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.max_by_key
            .max_by_key(|e| e.last_date.timestamp());
        // So, to overcome last problem, if first filtered message has same date than last_message, we replace last by first
        // As RSS feeds are supposed to put the latest emitted message in first position
        match last_message {
            Some(last) => if filtered_messages.len()>1 && filtered_messages[0].last_date==last.last_date {
                last_message = Some(&filtered_messages[0]);
            },
            _ => {}
        }
        
        let mut returned = self.clone();
        if settings.do_not_save {
            warn!("do_not_save is set. As a consequence, feed won't be updated");
        } else {
            match last_message {
                Some(message) => {
                    returned.last_updated = message.last_date;
                    returned.last_message = Some(message.id.clone());
                },
                _ => {}
            }
        }
        returned
    }
}
