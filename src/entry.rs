use chrono::{NaiveDateTime, DateTime, Utc};

use super::settings::*;
use super::extractable::*;

use atom_syndication::Entry;
use atom_syndication::Feed as SourceFeed;

impl Dated for Entry {
    fn last_date(&self)->NaiveDateTime {
        return self.updated().parse::<DateTime<Utc>>().unwrap().naive_utc();
    }
}

impl Extractable<SourceFeed> for Entry {
    fn get_content(&self, settings:&Settings) -> String {
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
        return text.to_owned();
    }
    fn get_title(&self, _settings:&Settings) -> String {
        return self.clone().title().to_owned();
    }
    fn get_id(&self, _settings:&Settings) -> String {
        return self.clone().id().to_owned();
    }
    fn get_links(&self, _settings:&Settings) -> Vec<String> {
        return self.clone().links().into_iter()
            .map(|l| l.href().to_owned())
            .collect();
    }
    fn get_authors(&self, feed:&SourceFeed, _settings:&Settings) -> Vec<String> {
        let message_authors:Vec<String> = self.clone().authors().into_iter()
            .map(|a| a.name().to_owned())
            .collect();
        if message_authors.is_empty() {
            let returned = vec![feed.title().to_owned()];
            return returned;
        } else {
            return message_authors;
        }
    }
}