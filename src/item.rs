use chrono::NaiveDateTime;

use super::settings::*;
use super::extractable::*;

use rss::Item;
use rss::Channel as SourceFeed;

use chrono::DateTime;

impl Dated for Item {
    fn last_date(&self)->NaiveDateTime {
        return DateTime::parse_from_rfc2822(&self.pub_date().unwrap()).unwrap().naive_utc();
    }

}

impl Extractable<SourceFeed> for Item {
    fn get_content(&self, settings:&Settings) -> String {
        let text = self.content().unwrap_or(self.description().unwrap_or(""));
        // First step is to fix HTML, so load it using html5ever 
        // (because there is no better html parser than a real browser one)
        // TODO implement image inlining
        return text.to_owned();
    }
    fn get_title(&self, _settings:&Settings) -> String {
        return self.clone().title().unwrap().to_owned();
    }
    fn get_id(&self, _settings:&Settings) -> String {
        return match self.guid() {
            Some(g) => g.value().to_owned(),
            _ => "no id".to_owned()
        };
    }
    fn get_links(&self, _settings:&Settings) -> Vec<String> {
        return match self.link() {
            Some(l) => vec![l.to_owned()],
            _ => vec![]
        }
    }
    fn get_authors(&self, feed:&SourceFeed, _settings:&Settings) -> Vec<String> {
        return match self.author() {
            Some(l) => vec![l.to_owned()],
            _ => vec![feed.title().to_owned()]
        }
    }
}