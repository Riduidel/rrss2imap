use std::path::PathBuf;
use std::path::Path;

use std::fs::File;
use std::fs;
use std::io::Read;

use serde_json;

use super::feed::Feed;
use super::config::Config;

#[derive(Debug, Deserialize, Serialize)]
pub struct Store {
    config:Config,
    feeds:Vec<Feed>
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
                config: Config {
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
        self.config.email = Some(email);
        self.save();
    }

    pub fn export(&self, file:Option<PathBuf>) {
        info!("exporting content not implemented");
    }

    pub fn import(&mut self, file:Option<PathBuf>) {
        info!("importing content not implemented");
    }

    // Add a feed to the feeds list and immediatly save the store
    pub fn add(&mut self, parameters:Vec<String>) {
        info!("adding \"{:?}\"", parameters);
        let to_add = Feed::from(parameters);
        // We never add the same feed twice. To ensure that, we check that no feed has the same url
        let tested = self.feeds.clone();
        let already_existing:Vec<&Feed> = tested.iter()
            .filter(|f| f.url==to_add.url)
            .collect();
        if already_existing.is_empty() {
            self.feeds.push(to_add);
        } else {
            panic!(format!("We already read this feed with the following configuration {:?}", already_existing));
        }
        self.save();
    }

    pub fn delete(&mut self, feed:u32) {
        let f = self.feeds.remove(feed as usize);
        self.save();
        info!("Removed {:?}", f);
    }

    pub fn reset(&mut self) {
        info!("reset not implemented");
    }

    pub fn run(&mut self) {
        info!("run not implemented");
    }

    pub fn list(&self) {
        let lines:Vec<String> = self.feeds.iter().enumerate()
            .map(|(i, f)| format!("{} : {} (to: {}) folder: {}", i, 
                f.url, 
                "TODO", //f.email.unwrap_or(self.config.email.unwrap()),
                "TODO"//f.folder.unwrap_or("TODO".to_string())
                ))
            .collect()
            ;
        println!("{}", &lines.join("\n"));
    }
}
