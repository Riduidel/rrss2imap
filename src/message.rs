use chrono::NaiveDateTime;

use super::feed::Feed;
use super::image_to_data;
use super::settings::*;
use mail_builder::MessageBuilder;
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

    fn build_from(&self, feed:&Feed, _settings:&Settings)->String {
        match &feed.config.from {
            Some(from) =>from.to_owned(),
            None => {
                if self.authors.is_empty() {
                    "what@what.com".to_owned()
                } else {
                    self.authors[0].to_owned()
                }
            }
        }
    }

    fn build_message(&self, feed: &Feed, settings: &Settings) -> Result<String, UnprocessableMessage> {
        let content = self.extract_content(feed, settings);
        debug!("===========================\nCreating message content\n{}\n===========================", content);
        let from = self.build_from(feed, settings);
        let _date = self.date_text();
        let to_addr = settings.config.email.as_ref().unwrap_or(&settings.email.user);
        let email = MessageBuilder::new()
            .from(from.as_str())
            .to(to_addr.as_str())
            .subject(str::replace(self.title.as_str(), "\n", ""))
            .html_body(content.as_str())
            .date(self.last_date.timestamp())
            .write_to_string()
            .unwrap();
        Ok(email)
    }

    /// Makes a valid HTML file out of the given Item.
    /// This method provides all the transformation that should happen
    fn extract_content(&self, _feed: &Feed, _settings: &Settings) -> String {
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
