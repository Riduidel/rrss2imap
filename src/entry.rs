use chrono::{NaiveDateTime, DateTime, Utc};

use super::settings::*;
use super::extractable::*;

use url::Url;
use atom_syndication::Entry;
use atom_syndication::Feed as SourceFeed;

impl Dated for Entry {
    fn last_date(&self)->NaiveDateTime {
        self.updated().parse::<DateTime<Utc>>().unwrap().naive_utc()
    }
}

impl Extractable<SourceFeed> for Entry {
    fn get_content(&self, _settings:&Settings) -> String {
        let text = match self.content() {
            Some(content) => content.value().unwrap(),
            None => match self.summary() {
                Some(summary) => summary,
                None => ""
            }
        };
        // First step is to fix HTML, so load it using html5ever 
        // (because there is no better html parser than a real browser one)
        // TODO implement image inlining
        text.to_owned()
    }
    fn get_title(&self, _settings:&Settings) -> String {
        self.clone().title().to_owned()
    }
    fn get_id(&self, _settings:&Settings) -> String {
        self.clone().id().to_owned()
    }
    fn get_links(&self, _settings:&Settings) -> Vec<String> {
        self.clone().links().iter()
            .map(|l| l.href().to_owned())
            .collect()
    }
    fn get_authors(&self, feed:&SourceFeed, _settings:&Settings) -> Vec<String> {
        let domain = find_domain(feed);
        // This is where we also transform author names into urls in order
        // to have valid email addresses everywhere
        let mut message_authors:Vec<String> = self.clone().authors().iter()
            .map(|a| a.name().to_owned())
            .collect();
        if message_authors.is_empty() {
            message_authors = vec![feed.title().to_owned()]
        }
        message_authors = message_authors.iter()
            .map(|author| (author, author
                                    .replace(" ", "_")))
            .map(|tuple| format!("{} <{}@{}>", tuple.0, tuple.1, domain))
            .collect();
        message_authors
    }
}

fn find_domain(feed:&SourceFeed) -> String {
    return feed.links().iter()
        .filter(|link| link.rel()=="self" || link.rel()=="alternate")
        .next()
        // Get the link
        .map(|link| link.href())
        // Transform it into an url
        .map(|href| Url::parse(href).unwrap())
        // then get host
        .map(|url| url.host_str().unwrap().to_string())
        // and return value
        .unwrap_or("todo.find.domain.rss".to_string())
        ;
}
