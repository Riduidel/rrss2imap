use chrono::NaiveDateTime;

use super::settings::*;
use super::extractable::*;

use rss::Item;
use rss::Channel as SourceFeed;

use chrono::DateTime;

impl Dated for Item {
    fn last_date(&self)->NaiveDateTime {
        if self.pub_date().is_some() {
            let pub_date = str::replace(self.pub_date().unwrap(), "-0000", "+0000");
            return DateTime::parse_from_rfc2822(&pub_date).unwrap_or_else(|e| panic!("pub_date for item {:?} (value is {:?}) can't be parsed. {:?}", 
                &self, pub_date, e)).naive_utc();
        } else if self.dublin_core_ext().is_some() && self.dublin_core_ext().unwrap().dates().len()>0 {
            let pub_date = &self.dublin_core_ext().unwrap().dates()[0];
            return DateTime::parse_from_rfc3339(&pub_date).unwrap_or_else(|e| panic!("dc:pub_date for item {:?} (value is {:?}) can't be parsed. {:?}", 
                &self, pub_date, e)).naive_utc();
        } else {
            panic!("feed item {:?} can't be parsed, as it doesn't have neither pub_date nor dc:pub_date", &self);
        }
    }

}

impl Extractable<SourceFeed> for Item {
    fn get_content(&self, _settings:&Settings) -> String {
        let text = self.content().unwrap_or_else(|| self.description().unwrap_or(""));
        // First step is to fix HTML, so load it using html5ever 
        // (because there is no better html parser than a real browser one)
        // TODO implement image inlining
        text.to_owned()
    }
    fn get_title(&self, _settings:&Settings) -> String {
        self.clone().title().unwrap().to_owned()
    }
    fn get_id(&self, _settings:&Settings) -> String {
        match self.guid() {
            Some(g) => g.value().to_owned(),
            _ => "no id".to_owned()
        }
    }
    fn get_links(&self, _settings:&Settings) -> Vec<String> {
        match self.link() {
            Some(l) => vec![l.to_owned()],
            _ => vec![]
        }
    }
    fn get_authors(&self, feed:&SourceFeed, _settings:&Settings) -> Vec<String> {
        match self.author() {
            Some(l) => vec![l.to_owned()],
            _ => vec![feed.title().to_owned()]
        }
    }
}