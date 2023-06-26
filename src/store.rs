extern crate directories;

use directories::ProjectDirs;
use tests_bin::unit_tests;
use std::path::{PathBuf, Path};

use std::fs;
use std::fs::File;
use std::io::Read;


use super::export;
use super::feed::Feed;
use super::import;
use super::settings::Settings;

use rayon::prelude::*;

use custom_error::custom_error;

custom_error!{pub UnusableStore
    IO{source:std::io::Error} = "input/output error",
    JsonParseError{source:serde_json::Error} = "Can't parse JSON content of store"
}

#[unit_tests("store.rs")]
/// Main application structure.
/// This structure is read/written from/to a JSON file
#[derive(Debug, Deserialize, Serialize)]
pub struct Store {
    /// Contains all application settings
    pub settings: Settings,
    /// Contains all feeds being read
    pub feeds: Vec<Feed>,
    #[serde(skip)]
    pub dirty:bool,
    #[serde(skip)]
    pub path: PathBuf
}

/// Name of the file from which config is read/written. As of today, this name is not expected to change.
pub const STORE: &str = "config.json";

/// Calculate the location of the `config.json` store file.
/// If `config.json` is found in the current directory, use it for backward
/// compatibility.  Otherwise, return a path inside the project directory
/// (~/.config/rrss2imap/ on Linux, system-specific on macOS and Windows).
pub fn find_store() -> PathBuf {
    let mut path = PathBuf::from(STORE);
    if !path.exists() {
        // The current directory takes precedence over project directory
        // for existing configurations for backward compatibility.
        if let Some(proj_dirs) = ProjectDirs::from("org", "Rrss2imap", "rrss2imap") {
            path = proj_dirs.config_dir().to_path_buf();
            path.push(STORE);
        }
    }
    path
}

impl Store {
    /// Initialize a Store object from a config file at the given path. If the
    /// config file does not exist, return a Store object with default values.
    pub fn load(path: &PathBuf) -> Result<Store,UnusableStore> {
        if path.exists() {
            info!("Reading config file {}", path.to_string_lossy());
            // First read the file
            let mut file = File::open(path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            // Then deserialize its content
            let mut store: Store =
                serde_json::from_str(&contents)?;
            store.path = path.to_owned();
            // And return it
            Ok(store)
        } else {
            info!("Using fresh config file {}", path.to_string_lossy());
            Ok(Store {
                settings: Settings::default(),
                feeds: vec![],
                dirty: false,
                path: path.to_owned()
            })
        }
    }

    /// Save all informations in the store file
    fn save(&self) {
        info!("Saving config file {}", self.path.to_string_lossy());
        let serialized = serde_json::to_string_pretty(self).expect("Can't serialize Store to JSON");
        let directory = self.path.parent().unwrap_or(Path::new("."));
        fs::create_dir_all(directory)
            .unwrap_or_else(|_| panic!("Unable to create directory for file {}", self.path.to_string_lossy()));
        fs::write(&self.path, serialized)
            .unwrap_or_else(|_| panic!("Unable to write file {}", self.path.to_string_lossy()));
    }

    /// Create a new configuration file with the given email.
    pub fn init_config(&mut self, email: String) {
        if self.path.exists() {
            warn!("Config file {} already exists, leaving it unchanged.", self.path.to_string_lossy());
        } else {
            println!("Config file {} created, please edit it to finish configuration.", self.path.to_string_lossy());
            self.settings.config.email = Some(email);
            self.dirty = true;
            self.save();
        }
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
        let path_to_write = file.expect("Can't export file if no file is given");
        warn!("exporting content to {:?}", path_to_write);
        export::export(&path_to_write, self);
        info!("exported feeds to {:?}", path_to_write);
    }

    /// Import rss feeds provided as an opml file
    /// see [import](rrss2imap::import::import) for implementation details
    pub fn import(&mut self, file: Option<PathBuf>) {
        let path_to_read = file.expect("Can't import file if no file is given");
        info!("importing content from {:?}", path_to_read);
        let count = self.feeds.len();
        import::import(&path_to_read, self);
        self.dirty = true;
        info!(
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
        info!("store has been cleared to contain only {:?}", self);
    }

    /// Run all rss to imap transformation
    /// Each feed is read and immediatly written in this thread.
    /// This should be rewritten to allow optimization/parallelism
    pub fn run(&mut self) {
        self.dirty = true;
        let feeds_length = self.feeds.len();
        // Initialize mail server before processing feeds
        self.feeds = self.feeds
            .par_iter().enumerate()
            .map(|element| (element.1, element.1.read(element.0, &feeds_length, )))
            .map(|(feed, messages)| feed.write_new_messages(&self.settings, messages))
            .collect::<Vec<Feed>>();
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
                error!("do_not_save flag is set in config.json. NOT SAVING {} !", self.path.to_string_lossy())
            } else {
                info!("store has been modified. Saving {} !", self.path.to_string_lossy());
                self.save();
            }
        }
    }
}
