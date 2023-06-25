use chrono::{DateTime, Utc, FixedOffset, NaiveDateTime};

use super::feed_errors::*;
use super::message::*;
use super::settings::*;
use atom_syndication::Entry as AtomEntry;
use atom_syndication::Feed as AtomFeed;
use rss::Channel as RssChannel;
use rss::Item as RssItem;
use url::Url;

use super::feed::*;
use super::feed_utils::*;

/// The reader trait allow reading data from a web source.
/// It is supposed to be derived for Rss and Atom, but it's only a try currently ...
pub trait Reader<EntryType, FeedType> {
    fn process_message(&self, feed:&Feed, settings:&Settings, message:&Message)->Message {
        Message {
            authors: message.authors.clone(),
            content: Message::get_processed_content(&message.content, feed, settings).unwrap(),
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
    fn find_new_messages(&self, feed:&Feed, sorted_messages:&[&Message])->(usize, usize, bool) {
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
                match &feed.last_message {
                    Some(id) => if id==&message.id {
                        tail = position; 
                        found = true;
                        break;
                    },
                    None => {}
                };
                if message.last_date<feed.last_updated {
                    tail = position; 
                    found = true;
                }
            }
        }
        (head, tail, found)
    }

    fn write_new_messages(&self, feed:&Feed, settings:&Settings, extracted:Vec<Result<Message, UnparseableFeed>>)->Feed {
        let sorted_messages:Vec<&Message> = extracted.iter()
            .filter_map(|e| e.as_ref().ok())
            .collect::<Vec<&Message>>();
        let (head, tail, found) = self.find_new_messages(feed, &sorted_messages);
        let filtered_messages:&[&Message] = if found {
            &sorted_messages[head..tail]
        } else {
            sorted_messages.as_slice()
        };

        // And write the messages into IMAP and the feed into JSON
        let written_messages:Vec<Message> = filtered_messages.iter()
            .map(|message| self.process_message(feed, settings, message))
            .inspect(|e| if !settings.do_not_save { e.write_to_imap(feed, settings) } )
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
                last_message = Some(filtered_messages[0].clone());
            },
            _ => {}
        }
        
        let mut returned = feed.clone();
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

    fn extract(&self, entry:&EntryType, source:&FeedType) -> Result<Message, UnparseableFeed>;
    fn read_feed_date(&self, source:&FeedType)->NaiveDateTime;

    fn extract_messages(&self, source:&FeedType)->Vec<Result<Message,UnparseableFeed>>;
    
    fn read(&self, feed:&Feed, source:&FeedType, settings:&Settings)->Feed {
        debug!("reading feed {}", &feed.url);
        let feed_date = self.read_feed_date(source);
        info!(
            "Feed date is {} while previous read date is {}",
            feed_date, feed.last_updated
        );
        let extracted:Vec<Result<Message, UnparseableFeed>> = self.extract_messages(source);

        let date_errors = extracted.iter()
            .filter(|e| e.is_err())
            .fold(0, |acc, _| acc + 1);
        if date_errors==0 {
            self.write_new_messages(feed, settings, extracted)
        } else {
            warn!("There were problems getting content from feed {}. It may not be complete ...
            I strongly suggest you enter an issue on GitHub by following this link
            https://github.com/Riduidel/rrss2imap/issues/new?title=Incorrect%20feed&body=Feed%20at%20url%20{}%20doesn't%20seems%20to%20be%20parseable", 
            feed.url, feed.url);
            feed.clone()
        }
    }
}

pub struct AtomReader {}

impl AtomReader {
    fn extract_authors_from_atom(entry: &AtomEntry, feed: &AtomFeed) -> Vec<(String, String)> {
        let domain = AtomReader::find_atom_domain(feed);
        // This is where we also transform author names into urls in order
        // to have valid email addresses everywhere
        let mut message_authors: Vec<String> = entry
            .authors()
            .iter()
            .map(|a| a.name().to_owned())
            .collect();
        if message_authors.is_empty() {
            message_authors = vec![feed.title().to_owned().to_string()]
        }
        sanitize_message_authors(message_authors, domain)
    }
    
    fn find_atom_domain(feed: &AtomFeed) -> String {
        return feed
            .links()
            .iter()
            .filter(|link| link.rel() == "self" || link.rel() == "alternate").find(|link| !link.href().is_empty())
            // Get the link
            .map(|link| link.href())
            // Transform it into an url
            .map(|href| Url::parse(href).unwrap())
            // then get host
            .map(|url| url.host_str().unwrap().to_string())
            // and return value
            .unwrap_or("todo.find.domain.rss".to_string());
    }
}

impl Reader<AtomEntry, AtomFeed> for AtomReader {
    fn extract(&self, entry: &AtomEntry, source: &AtomFeed) -> Result<Message, UnparseableFeed> {
        info!("Reading atom entry {} from {:?}", entry.id(), entry.links());
        let authors = AtomReader::extract_authors_from_atom(entry, source);
        let last_date = entry
            .updated()
            .naive_utc();
        let content = match entry.content() {
            Some(content) => content.value().unwrap(),
            None => match entry.summary() {
                Some(text)=> text.as_str(),
                None=>""
            }
        }
        .to_owned();
        let message = Message {
            authors,
            content,
            id: entry.id().to_owned(),
            last_date,
            links: entry.links().iter().map(|l| l.href().to_owned()).collect(),
            title: entry.title().as_str().to_string()
        };
        Ok(message)
    }

    fn read_feed_date(&self, source:&AtomFeed)->NaiveDateTime {
        source.updated().naive_utc()
    }

    fn extract_messages(&self, source:&AtomFeed)->Vec<Result<Message, UnparseableFeed>> {
        source.entries()
            .iter()
            .map(|e| self.extract(e, source))
            .collect()
    }
}

pub struct RssReader {}

impl RssReader {
    fn extract_authors_from_rss(entry: &RssItem, feed: &RssChannel) -> Vec<(String, String)> {
        let domain = RssReader::find_rss_domain(feed);
        // This is where we also transform author names into urls in order
        // to have valid email addresses everywhere
        let message_authors: Vec<String>;
        match entry.author() {
            Some(l) => message_authors = vec![l.to_owned()],
            _ => message_authors = vec![feed.title().to_owned()],
        }
        sanitize_message_authors(message_authors, domain)
    }
    fn find_rss_domain(feed: &RssChannel) -> String {
        return Some(feed.link())
            .map(|href| Url::parse(href).unwrap())
            // then get host
            .map(|url| url.host_str().unwrap().to_string())
            // and return value
            .unwrap_or("todo.find.domain.atom".to_string());
    }

    fn try_hard_to_parse(date:String) -> Result<DateTime<FixedOffset>, UnparseableFeed> {
        let parsed = rfc822_sanitizer::parse_from_rfc2822_with_fallback(&date);
        if parsed.is_ok() {
            Ok(parsed?)
        } else {
            let retry = DateTime::parse_from_rfc3339(&date);
            if retry.is_ok() {
                Ok(retry?)
            } else {
                Err(UnparseableFeed::DateIsNeitherRFC2822NorRFC3339 {value:date})
            }
        }
    }
    
    fn extract_date_from_rss(entry: &RssItem, feed: &RssChannel) -> Result<DateTime<FixedOffset>, UnparseableFeed> {
        if entry.pub_date().is_some() {
            let mut pub_date = entry.pub_date().unwrap().to_owned();
            pub_date = pub_date.replace("UTC", "UT");
            RssReader::try_hard_to_parse(pub_date)
        } else if entry.dublin_core_ext().is_some()
            && !entry.dublin_core_ext().unwrap().dates().is_empty()
        {
            let pub_date = &entry.dublin_core_ext().unwrap().dates()[0];
            Ok(DateTime::parse_from_rfc3339(pub_date)?)
        } else {
            debug!("feed item {:?} date can't be parsed, as it doesn't have neither pub_date nor dc:pub_date. We will replace it with feed date if possible",
                &entry.link()
            );
            if feed.pub_date().is_some() {
                let pub_date = feed.pub_date().unwrap().to_owned();
                RssReader::try_hard_to_parse(pub_date)
            } else if feed.last_build_date().is_some() {
                let last_pub_date = feed.last_build_date().unwrap().to_owned();
                RssReader::try_hard_to_parse(last_pub_date)
            } else {
                Ok(DateTime::<FixedOffset>::from_utc(
                    Feed::at_epoch(), 
                    FixedOffset::east_opt(0).unwrap()))
            }
        }
    }
}

impl Reader<RssItem, RssChannel> for RssReader {
    fn extract(&self, entry: &RssItem, source: &RssChannel) -> Result<Message, UnparseableFeed> {
        info!("Reading RSS entry {:?} from {:?}", entry.guid(), entry.link());
        let authors = RssReader::extract_authors_from_rss(entry, source);
        let content = entry
            .content()
            .unwrap_or_else(|| entry.description().unwrap_or(""))
            // First step is to fix HTML, so load it using html5ever
            // (because there is no better html parser than a real browser one)
            // TODO implement image inlining
            .to_owned();
        let links = match entry.link() {
            Some(l) => vec![l.to_owned()],
            _ => vec![],
        };
        let id = if links.is_empty() {
            match entry.guid() {
                Some(g) => g.value().to_owned(),
                _ => "no id".to_owned(),
            }
        } else {
            links[0].clone()
        };
        let last_date = RssReader::extract_date_from_rss(entry, source);
        let message = Message {
            authors,
            content,
            id,
            last_date: last_date?.naive_utc(),
            links,
            title: entry.title().unwrap_or("").to_owned(),
        };
        Ok(message)
    }

    fn extract_messages(&self, source:&RssChannel)->Vec<Result<Message, UnparseableFeed>> {
        source.items()
            .iter()
            .map(|e| self.extract(e, source))
            .collect()
    }

    fn read_feed_date(&self, source:&RssChannel)->NaiveDateTime {
        let n = Utc::now();
        let feed_date_text = match source.pub_date() {
            Some(p) => p.to_owned(),
            None => match source.last_build_date() {
                Some(l) => l.to_owned(),
                None => n.to_rfc2822(),
            },
        };
        DateTime::parse_from_rfc2822(&feed_date_text)
            .unwrap()
            .naive_utc()
        
    }
}
