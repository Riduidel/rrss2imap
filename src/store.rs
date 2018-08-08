use std::path::PathBuf;

pub struct Store {

}

impl Store {
    /// Loads the FeedStore object.
    /// This requires creating (if it doesn't exist) the config.xml file
    /// And filling it with useful content
    pub fn load() -> Store {
        Store {}
    }

    pub fn set_email(&mut self, email:String) {
        info!("setting email to {} not implemented", email);
    }

    pub fn export(&self, file:Option<PathBuf>) {
        info!("exporting content not implemented");
    }

    pub fn import(&mut self, file:Option<PathBuf>) {
        info!("importing content not implemented");
    }

    pub fn add(&mut self, url:String, email:String, folder:String) {
        info!("adding {} for {} in folder {} not implemented", url, email, folder);
    }

    pub fn delete(&mut self, feed:u32) {
        info!("deleting {} not implemented", feed);
    }

    pub fn reset(&mut self) {
        info!("reset not implemented");
    }

    pub fn run(&mut self) {
        info!("run not implemented");
    }

    pub fn list(&self, ) {
        info!("list not implemented");
    }
}
