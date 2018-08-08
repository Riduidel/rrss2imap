#[macro_use]

use std::path::PathBuf;

pub struct FeedStore {

}

impl FeedStore {
    /// Loads the FeedStore object.
    /// This requires creating (if it doesn't exist) the config.xml file
    /// And filling it with useful content
    pub fn load() -> FeedStore {
        FeedStore {}
    }

    pub fn set_email(&mut self, email:String) {
        panic!(format!("setting email to {} not implemented", email));
    }

    pub fn export(&self, file:Option<PathBuf>) {
        panic!("exporting content not implemented");
    }

    pub fn import(&mut self, file:Option<PathBuf>) {
        panic!("importing content not implemented");
    }

    pub fn add(&mut self, url:String, email:String, folder:String) {
        panic!(format!("adding {} for {} in folder {} not implemented", url, email, folder));
    }

    pub fn delete(&mut self, feed:u32) {
        panic!(format!("deleting {} not implemented", feed));
    }

    pub fn reset(&mut self) {
        panic!("reset not implemented");
    }

    pub fn run(&mut self) {
        panic!("run not implemented");
    }

    pub fn list(&self, ) {
        panic!("list not implemented");
    }
}
