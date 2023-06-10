use chrono::NaiveDateTime;
use chrono::format::format;

use super::feed::Feed;
use super::image_to_data;
use super::settings::*;

use emailmessage::{header, Message as Email, SinglePart};
use emailmessage::header::EmailDate;
use emailmessage::Mailbox;

use custom_error::custom_error;

custom_error!{pub UnprocessableMessage
    CantPutDateInMessage{ value:String } = "EmailMessage can't parse date from {value}",
    CantPutFirstAuthorInMessage { value:String } = "Unable to parse first author {value}.
    Please consider adding in feed config the \"from\": ... field",
    CantWriteTransformedMessage = "Can't re-write transformed message after image Base64'ing"
}

///
/// Structure for storing message data prior to having these messages written to IMAP.
/// This structure serves as a common interface for Item/Entry
pub struct Message {
    /// List of message authors
    pub authors: Vec<String>,
    /// Message content. Image extraction should happen BEFORE that storage.
    pub content: String,
    /// Message id
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
        let content = self.extract_content(feed, settings);
        debug!("===========================\nCreating message content\n{}\n===========================", content);
        let date:Result<EmailDate, _> = self.date_text().parse();
        if date.is_err() {
            return Err(UnprocessableMessage::CantPutDateInMessage { value : self.date_text() });
        }
        let to_addr = settings.config.email.as_ref().unwrap_or(&settings.email.user);
        let mut builder = Email::builder()
            .subject(&*self.title)
            .date(date.unwrap())
            .to(to_addr.parse().unwrap())
            ;

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
    fn extract_content(&self, feed: &Feed, settings: &Settings) -> String {
        let style = include_str!("message.css");
        let title = format!("<h1 class=\"header\"><a href=\"{}\">{}</a></h1>",
            self.id,
            self.title);
        let body = format!("<div id=\"body\">{}</div>", self.content);
        let links = self.links.iter()
                    .map(|l| format!("<p class=\"footer\">URL: <a href=\"{}\">{}</a></p>", l, l))
                    .collect::<Vec<String>>()
                    .join("\n")
                    ;
        format!("
        <html>
        <head>
            <meta http-equiv=\"Content-Type\" content=\"text/html\">
            <style>
                {}
            </style>
        </head>
    
        <body>
            <div id=\"entry\">
                {}
                {}
                {}
            </div>
        </body>
    </html>
        ",
            style,
            title,
            body,
            links)
    }

    ///
    /// Process the feed effective content.
    /// This should allow
    /// * image transformation into base64 when needed
    ///
    pub fn get_processed_content(html_content:&String, feed: &Feed, settings: &Settings) -> Result<String, UnprocessableMessage> {
        if feed.config.inline_image_as_data || settings.config.inline_image_as_data {
            match image_to_data::transform(html_content) {
                Ok(transformed_html_content) => Ok(transformed_html_content),
                Err(_) => Err(UnprocessableMessage::CantWriteTransformedMessage)
            }
        } else {
            Ok(html_content.clone())
        }
    }

    fn date_text(&self) -> String {
        self.last_date
            .format("%a, %d %b %Y %H:%M:%S -0000")
            .to_string()
    }
}
