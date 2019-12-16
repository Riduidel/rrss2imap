use std::path::Path;
use std::path::PathBuf;

use std::fs;
use std::fs::File;
use std::io::Read;

use serde_json;

use futures::stream::StreamExt;
use futures::stream::futures_unordered::FuturesUnordered;

use super::export;
use super::feed::Feed;
use super::import;
use super::settings::Settings;

use custom_error::custom_error;

custom_error!{pub UnusableStore
    IO{source:std::io::Error} = "input/output error",
    JsonParseError{source:serde_json::Error} = "Can't parse JSON content of store"
}


/// Main application structure.
/// This structure is read/written from/to a JSON file
#[derive(Debug, Deserialize, Serialize)]
pub struct Store {
    /// Contains all application settings
    pub settings: Settings,
    /// Contains all feeds being read
    pub feeds: Vec<Feed>,
    #[serde(skip)]
    pub dirty:bool
}

/// Name of the file from which config is read/written. As of today, this name is not expected to change.
pub const STORE: &str = "config.json";

impl Store {
    /// Loads the FeedStore object.
    /// This requires creating (if it doesn't exist) the config.xml file
    /// And filling it with useful content
    pub fn load() -> Result<Store,UnusableStore> {
        let path = Path::new(STORE);
        if path.exists() {
            // First read the file
            let mut file =
                File::open(STORE)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            // Then deserialize its content
            let store: Store =
                serde_json::from_str(&contents)?;
            // And return it
            return Ok(store);
        } else {
            return Ok(Store {
                settings: Settings::default(),
                feeds: vec![],
                dirty: false
            });
        }
    }

    /// Save all informations in the store file
    fn save(&self) {
        let serialized = serde_json::to_string_pretty(self).expect("Can't serialize Store to JSON");
        fs::write(STORE, serialized).unwrap_or_else(|_| panic!("Unable to write file {}", STORE));
    }

    /// Set a new value for email and save file (prior to obviously exiting)
    pub fn set_email(&mut self, email: String) {
        self.settings.config.email = Some(email);
        self.dirty = true;
        self.save();
    }

    /// Exports config into an OPML file
    /// see [export](rrss2imap::export::export) for implementation details
    pub fn export(&self, file: Option<PathBuf>) {
        let path_to_write = file.expect("Can't expport file if no file is given");
        warn!("exporting content to {:?}", path_to_write);
        export::export(&path_to_write, self);
        warn!("exported feeds to {:?}", path_to_write);
    }

    /// Import rss feeds provided as an opml file
    /// see [import](rrss2imap::import::import) for implementation details
    pub fn import(&mut self, file: Option<PathBuf>) {
        let path_to_read = file.expect("Can't import file if no file is given");
        warn!("importing content from {:?}", path_to_read);
        let count = self.feeds.len();
        import::import(&path_to_read, self);
        self.dirty = true;
        warn!(
            "imported {} feeds from {:?}",
            self.feeds.len() - count,
            path_to_read
        );
    }

    /// Add a feed to the feeds list and immediatly save the store.
    pub fn add(&mut self, url:Option<String>, email:Option<String>, destination:Option<String>, inline:bool, parameters: Vec<String>) {
        let to_add:Feed = if url.is_some() {
            Feed::from_all(url, email, destination, inline)
        } else {
            Feed::from_vec(parameters)
        };
        info!("adding \"{:?}\"", to_add);
        self.add_feed(to_add);
        self.dirty = true;
    }

    /// Delete the feed which id is given as parameter.
    /// The use of a number is a compatibility requirement
    pub fn delete(&mut self, feed: u32) {
        let f = self.feeds.remove(feed as usize);
        self.dirty = true;
        info!("Removed {:?}", f);
    }

    /// Reset the config file by removing all feeds and config
    pub fn reset(&mut self) {
        self.feeds.clear();
        self.settings.config.clear();
        self.dirty = true;
        info!("store have been cleared to contain only {:?}", self);
    }

    /// Run all rss to imap transformation
    /// Each feed is read and immediatly written in this thread.
    /// This should be rewritten to allow optimization/parallelism
    pub async fn run(&mut self) {
        self.dirty = true;
        let client = reqwest::Client::builder()
            .build().unwrap();
        // Initialize mail server before processing feeds
        self.feeds = self.feeds
            .iter()
            .map(|f| f.read(&client, &self.settings))
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<Feed>>()
            .await;
    }

    /// Prints all the feeds to stdout.
    /// This is done in a way compatible with rss2imap original layout.
    /// As a consequence, new elements (like image inlining) are not visible
    pub fn list(&self) {
        let lines: Vec<String> = self
            .feeds
            .iter()
            .enumerate()
            .map(|(i, f)| format!("{} : {}", i, f.to_string(&self.settings.config)))
            .collect();
        println!("{}", &lines.join("\n"));
    }

    /// If the feed url is not already in the store, adds it
    pub fn add_feed(&mut self, to_add: Feed) {
        // We never add the same feed twice. To ensure that, we check that no feed has the same url
        let tested = self.feeds.clone();
        let already_existing: Vec<&Feed> = tested.iter().filter(|f| f.url == to_add.url).collect();
        if already_existing.is_empty() {
            self.feeds.push(to_add);
        } else {
            error!(
                "We already read this feed with the following configuration {:?}",
                already_existing
            );
        }
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        if self.dirty {
            if self.settings.do_not_save {
                error!("do_not_save flag is set in config.json. NOT SAVING!")
            } else {
                info!("store has been modified. Saving !");
                self.save();
            }
        }
    }
}