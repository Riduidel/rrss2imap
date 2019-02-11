use chrono::NaiveDateTime;

use super::settings::*;
use super::feed::*;
use super::config::*;

use tera::Tera;

lazy_static! {
    pub static ref TERA:Tera = {
        let mut tera = compile_templates!("templates/*");
        tera.autoescape_on(vec![]);
        tera
    };
}

/// Provides last date for an element
pub trait Dated {
    fn last_date(&self)->NaiveDateTime;
}

pub trait Extractable<SourceFeed> {
    fn get_content(&self, settings:&Settings) -> String;
    fn get_title(&self, settings:&Settings) -> String;
    fn get_link(&self, settings:&Settings) -> String;
    fn get_author(&self, feed:&SourceFeed, settings:&Settings) -> String;
    /// Makes a valid HTML file out of the given Item.
    /// This method provides all the transformation that should happen
    fn extract_content(&self, feed:&SourceFeed, settings:&Settings) -> String;

    fn write_to_imap(&self, feed:&Feed, source:&SourceFeed, settings:&Settings, config:&Config, email:&mut Imap);

}
