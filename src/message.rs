use chrono::NaiveDateTime;

use super::feed::Feed;
use super::image_to_data;
use super::settings::*;
use tera::Context;
use tera::Tera;

use kuchiki::traits::*;

use emailmessage::{header, Message as Email, SinglePart};
use emailmessage::header::EmailDate;
use emailmessage::Mailbox;

use custom_error::custom_error;

custom_error!{UnprocessableMessage
    CantPutDateInMessage{ value:String } = "EmailMessage can't parse date from {value}",
    CantPutFirstAuthorInMessage { value:String } = "Unable to parse first author {value}.
    Please consider adding in feed config the \"from\": ... field",
    CantWriteTransformedMessage = "Can't re-write transformed message after image Base64'ing"
}


lazy_static! {
    pub static ref TERA: Tera = {
        let mut tera = compile_templates!("templates/*");
        let message = include_str!("../templates/message.html");
        tera.add_raw_template("message.html", message).expect("There should be a message.html template");
        tera.autoescape_on(vec![]);
        tera
    };
}

///
/// Structure for storing message data prior to having these messages written to IMAP.
/// This structure serves as a common interface for Item/Entry
pub struct Message {
    pub authors: Vec<String>,
    pub content: String,
    pub id: String,
    pub last_date: NaiveDateTime,
    pub links: Vec<String>,
    pub title: String,
}

impl Message {
    pub fn write_to_imap(&self, feed: &Feed, settings: &Settings) {
        let folder = feed.config.get_folder(&settings.config);
        let content = self.build_message(feed, settings);
        match content {
            Ok(text) => {
                debug!("===========================\nWriting message content to IMAP\n{}\n===========================", 
                    text);
                match settings.email.append(&folder, &text) {
                    Ok(_) => debug!("Successfully written {}", self.title),
                    Err(e) => error!(
                        "{}\nUnable to select mailbox {}. Item titled {} won't be written",
                        e, &folder, self.title
                    ),
                }
            },
            Err(error) => {
                warn!("Couldn(t write message {:?} from feed {} due to {}", self.links, feed.url, error);
            }
        }
    }

    fn build_message(&self, feed: &Feed, settings: &Settings) -> Result<String, UnprocessableMessage> {
        let content = self.extract_content(feed, settings)?;
        debug!("===========================\nCreating message content\n{}\n===========================", content);
        let date:Result<EmailDate, _> = self.date_text().parse();
        if date.is_err() {
            return Err(UnprocessableMessage::CantPutDateInMessage { value : self.date_text() });
        }
        let mut builder = Email::builder()
            .subject(&*self.title)
            .date(date.unwrap());

        match &feed.config.from {
            Some(from) => {
                builder = builder.from(from.parse().unwrap());
            }
            None => {
                if self.authors.is_empty() {
                    builder = builder.from("what@what.com".parse().unwrap());
                } else {
                    let first_author = &self.authors[0];
                    let parsed_first_author:Result<Mailbox, _> = first_author.parse();
                    if parsed_first_author.is_err() {
                        return Err(UnprocessableMessage::CantPutFirstAuthorInMessage { value : first_author.clone() });
                    }
                    builder = builder.from(parsed_first_author.unwrap());
                }
            }
        }

        let email: Email<SinglePart<String>> = builder.mime_body(
            SinglePart::builder()
                .header(header::ContentType(
                    "text/html; charset=utf8".parse().unwrap(),
                ))
                .header(header::ContentTransferEncoding::QuotedPrintable)
                .body(content),
        );
        Ok(email.to_string())
    }

    /// Makes a valid HTML file out of the given Item.
    /// This method provides all the transformation that should happen
    fn extract_content(&self, feed: &Feed, settings: &Settings) -> Result<String, UnprocessableMessage> {
        Ok(TERA.render("message.html", &self.build_context(feed, settings)?)
            .unwrap())
    }

    ///
    /// Process the feed effective content.
    /// This should allow
    /// * image transformation into base64 when needed
    ///
    fn get_processed_content(&self, feed: &Feed, settings: &Settings) -> Result<String, UnprocessableMessage> {
        if feed.config.inline_image_as_data || settings.config.inline_image_as_data {
            let mut document = kuchiki::parse_html().one(self.content.clone());
            // So, take content, pass it through html5ever (thanks to select, and transform each image !)
            debug!("We should inline image as base64 data for {}", self.id);
            document = image_to_data::transform(document, feed, settings);
            let mut bytes = vec![];
            if document.serialize(&mut bytes).is_err() {
                return Err(UnprocessableMessage::CantWriteTransformedMessage);
            }
            return Ok(String::from_utf8(bytes).unwrap());
        } else {
            return Ok(self.content.clone());
        }
    }

    fn build_context(&self, feed: &Feed, settings: &Settings) -> Result<Context, UnprocessableMessage> {
        let mut context = Context::new();
        let parsed:Result<String, UnprocessableMessage> = self.get_processed_content(feed, settings);
        match parsed {
            Ok(text) => {
                context.insert("feed_entry", &text);
                context.insert("links", &self.links);
                context.insert("id", &self.id);
                context.insert("title", &self.title);
                context.insert("from", &self.authors);
                context.insert("to", &feed.config.get_email(&settings.config));
                context.insert("date", &self.date_text());
                return Ok(context)
            },
            Err(error) => return Err(error)
        }
    }

    fn date_text(&self) -> String {
        self.last_date
            .format("%a, %d %b %Y %H:%M:%S -0000")
            .to_string()
    }
}
