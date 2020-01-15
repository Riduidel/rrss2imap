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

    fn write_new_messages(&self, feed:&Feed, settings:&Settings, extracted:Vec<Result<Message, UnparseableFeed>>)->Feed {
        return extracted.iter()
            .filter(|e| e.is_ok())
            .map(|e| e.as_ref().unwrap())
            .filter(|m| m.last_date>feed.last_updated)
            .map(|message| self.process_message(feed, settings, message))
            .inspect(|e| if !settings.do_not_save { e.write_to_imap(&feed, settings) } )
            .map(|e| e.last_date)
            .max()
            .map_or_else(
                || feed.clone(),
                |feed_date| Feed {
                    url: feed.url.clone(),
                    config: feed.config.clone(),
                    last_updated: if settings.do_not_save {
                        warn!("do_not_save is set. As a consequence, feed won't be updated");
                        feed.last_updated
                    } else {
                        feed_date
                    }
                }
            );
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
            return self.write_new_messages(feed, settings, extracted);
        } else {
            warn!("There were problems getting content from feed {}. It may not be complete ...
            I strongly suggest you enter an issue on GitHub by following this link
            https://github.com/Riduidel/rrss2imap/issues/new?title=Incorrect%20feed&body=Feed%20at%20url%20{}%20doesn't%20seems%20to%20be%20parseable", 
            feed.url, feed.url);
            return feed.clone();
        }
    }
}

pub struct AtomReader {}

impl AtomReader {
    fn extract_authors_from_atom(entry: &AtomEntry, feed: &AtomFeed) -> Vec<String> {
        let domain = AtomReader::find_atom_domain(feed);
        // This is where we also transform author names into urls in order
        // to have valid email addresses everywhere
        let mut message_authors: Vec<String> = entry
            .authors()
            .iter()
            .map(|a| a.name().to_owned())
            .collect();
        if message_authors.is_empty() {
            message_authors = vec![feed.title().to_owned()]
        }
        sanitize_message_authors(message_authors, domain)
    }
    
    fn find_atom_domain(feed: &AtomFeed) -> String {
        return feed
            .links()
            .iter()
            .filter(|link| link.rel() == "self" || link.rel() == "alternate")
            .filter(|link| !link.href().is_empty())
            .next()
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
        let authors = AtomReader::extract_authors_from_atom(entry, source);
        let last_date = entry
            .updated()
            .parse::<DateTime<Utc>>()
            .unwrap()
            .naive_utc();
        let content = match entry.content() {
            Some(content) => content.value().unwrap(),
            None => match entry.summary() {
                Some(summary) => summary,
                None => "",
            },
        }
        .to_owned();
        let message = Message {
            authors: authors,
            content: content,
            id: entry.id().to_owned(),
            last_date: last_date,
            links: entry.links().iter().map(|l| l.href().to_owned()).collect(),
            title: entry.title().to_owned(),
        };
        return Ok(message);
    }

    fn read_feed_date(&self, source:&AtomFeed)->NaiveDateTime {
        let feed_date_text = source.updated();
        return if feed_date_text.is_empty() {
            Feed::at_end_of_universe()
        } else {
            feed_date_text.parse::<DateTime<Utc>>().unwrap().naive_utc()
        };
    }

    fn extract_messages(&self, source:&AtomFeed)->Vec<Result<Message, UnparseableFeed>> {
        source.entries()
            .iter()
            .map(|e| self.extract(e, &source))
            .collect()
    }
}

pub struct RssReader {}

impl RssReader {
    fn extract_authors_from_rss(entry: &RssItem, feed: &RssChannel) -> Vec<String> {
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
            return Ok(parsed?);
        } else {
            let retry = DateTime::parse_from_rfc3339(&date);
            if retry.is_ok() {
                return Ok(retry?);
            } else {
                return Err(UnparseableFeed::DateIsNeitherRFC2822NorRFC3339 {value:date});
            }
        }
    }
    
    fn extract_date_from_rss(entry: &RssItem, feed: &RssChannel) -> Result<DateTime<FixedOffset>, UnparseableFeed> {
        if entry.pub_date().is_some() {
            let mut pub_date = entry.pub_date().unwrap().to_owned();
            pub_date = pub_date.replace("UTC", "UT");
            return RssReader::try_hard_to_parse(pub_date);
        } else if entry.dublin_core_ext().is_some()
            && entry.dublin_core_ext().unwrap().dates().len() > 0
        {
            let pub_date = &entry.dublin_core_ext().unwrap().dates()[0];
            return Ok(DateTime::parse_from_rfc3339(&pub_date)?);
        } else {
            error!("feed item {:?} date can't be parsed, as it doesn't have neither pub_date nor dc:pub_date. We will replace it with feed date if possible",
                &entry.link()
            );
            if feed.pub_date().is_some() {
                let pub_date = feed.pub_date().unwrap().to_owned();
                return RssReader::try_hard_to_parse(pub_date);
            } else if feed.last_build_date().is_some() {
                let last_pub_date = feed.last_build_date().unwrap().to_owned();
                return RssReader::try_hard_to_parse(last_pub_date);
            } else {
                return Err(UnparseableFeed::NoDateFound);
            }
        }
    }
}

impl Reader<RssItem, RssChannel> for RssReader {
    fn extract(&self, entry: &RssItem, source: &RssChannel) -> Result<Message, UnparseableFeed> {
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
            authors: authors,
            content: content,
            id: id,
            last_date: last_date?.naive_utc(),
            links: links,
            title: entry.title().unwrap_or("").to_owned(),
        };
        return Ok(message);
    }

    fn extract_messages(&self, source:&RssChannel)->Vec<Result<Message, UnparseableFeed>> {
        source.items()
            .iter()
            .map(|e| self.extract(e, &source))
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
        return DateTime::parse_from_rfc2822(&feed_date_text)
            .unwrap()
            .naive_utc();
        
    }
}
