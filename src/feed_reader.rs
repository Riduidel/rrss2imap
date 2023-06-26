use chrono::{DateTime, Utc, FixedOffset, NaiveDateTime};

use super::feed_errors::*;
use super::message::*;
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
    fn extract(&self, entry:&EntryType, source:&FeedType) -> Result<Message, UnparseableFeed>;
    fn read_feed_date(&self, source:&FeedType)->NaiveDateTime;

    fn extract_messages(&self, source:&FeedType)->Vec<Result<Message,UnparseableFeed>>;
    
    fn read<'lifetime>(&self, feed:&'lifetime Feed, source:&FeedType)->Vec<Message> {
        debug!("reading feed {}", &feed.url);
        let feed_date = self.read_feed_date(source);
        info!(
            "Feed date is {} while previous read date is {}",
            feed_date, feed.last_updated
        );
        let extracted:Vec<Result<Message, UnparseableFeed>> = self.extract_messages(source);

        let messages:Result<Vec<Message>, UnparseableFeed>  = extracted.into_iter().collect();
        messages.unwrap_or(vec![])
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
