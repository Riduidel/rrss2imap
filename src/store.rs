use std::path::PathBuf;
use std::path::Path;

use std::fs::File;
use std::fs;
use std::io::Read;

use serde_json;

use super::feed::Feed;
use super::config::Config;
use super::import;
use super::export;

#[derive(Debug, Deserialize, Serialize)]
pub struct Store {
    pub default:Config,
    pub feeds:Vec<Feed>
}

const STORE:&str = "config.json";

impl Store {
    /// Loads the FeedStore object.
    /// This requires creating (if it doesn't exist) the config.xml file
    /// And filling it with useful content
    pub fn load() -> Store {
        let path = Path::new(STORE);
        if path.exists() {
            // First read the file
            let mut file = File::open(STORE).expect(&format!("Unable to open file {}", STORE));
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect(&format!("Unable to read file {}", STORE));
            // Then deserialize its content
            let store:Store = serde_json::from_str(&contents).expect("Can't serialize Store to JSON");
            // And return it
            return store;
        } else {
            Store {
                default: Config {
                    email:None,
                    folder:None
                },
                feeds:vec!()
            }
        }
    }

    /// Save all informations in the store file
    fn save(&self) {
        let serialized = serde_json::to_string_pretty(self).expect("Can't serialize Store to JSON");
        fs::write(STORE, serialized).expect(&format!("Unable to write file {}", STORE));
    }

    /// Set a new value for email and save file (prior to obviously exiting)
    pub fn set_email(&mut self, email:String) {
        self.default.email = Some(email);
        self.save();
    }

    pub fn export(&self, file:Option<PathBuf>) {
        let path_to_write = file.expect("Can't expport file if no file is given");
        warn!("exporting content to {:?}", path_to_write);
        export::export(&path_to_write, self);
        warn!("exported feeds to {:?}", path_to_write);
    }

    /// Import rss feeds provided as an opml file
    pub fn import(&mut self, file:Option<PathBuf>) {
        let path_to_read = file.expect("Can't import file if no file is given");
        warn!("importing content from {:?}", path_to_read);
        let count = self.feeds.len();
        import::import(&path_to_read, self);
        self.save();
        warn!("imported {} feeds from {:?}", self.feeds.len()-count, path_to_read);
    }

    // Add a feed to the feeds list and immediatly save the store
    pub fn add(&mut self, parameters:Vec<String>) {
        info!("adding \"{:?}\"", parameters);
        let to_add = Feed::from(parameters);
        self.add_feed(to_add);
        self.save();
    }

    pub fn delete(&mut self, feed:u32) {
        let f = self.feeds.remove(feed as usize);
        self.save();
        info!("Removed {:?}", f);
    }

    pub fn reset(&mut self) {
        self.feeds.clear();
        self.default.clear();
        self.save();
        info!("store have been cleared to contain only {:?}", self);
    }

    pub fn run(&mut self) {
        error!("run not implemented");
    }

    pub fn list(&self) {
        let lines:Vec<String> = self.feeds.iter().enumerate()
            .map(|(i, f)| format!("{} : {}", i, f.to_string(&self.default)))
            .collect()
            ;
        println!("{}", &lines.join("\n"));
    }

    pub fn add_feed(&mut self, to_add:Feed) {
        // We never add the same feed twice. To ensure that, we check that no feed has the same url
        let tested = self.feeds.clone();
        let already_existing:Vec<&Feed> = tested.iter()
            .filter(|f| f.url==to_add.url)
            .collect();
        if already_existing.is_empty() {
            self.feeds.push(to_add);
        } else {
            error!("We already read this feed with the following configuration {:?}", already_existing);
        }
    }
}
