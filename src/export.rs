use std::path::PathBuf;

use std::fs;

use std::collections::HashMap;

use super::config::Config;
use super::feed::Feed;
use super::store::Store;

use treexml::*;

pub fn export(to_file: &PathBuf, to_store: &Store) {
    // First group feeds per storage folder
    let grouped = group_feeds(to_store);
    // Then write this map of lists
    write(to_file, grouped);
}

fn group_feeds(to_store: &Store) -> HashMap<String, Vec<Feed>> {
    return to_store.feeds.iter().fold(HashMap::new(), |mut map, feed| {
        let feed = feed.clone();
        let folder = feed.clone().config.get_folder(&to_store.default);
        if !map.contains_key(&folder) {
            map.insert(folder.clone(), vec![]);
        }
        let mut updated = vec![feed];
        updated.append(map.get_mut(&folder).unwrap());
        map.insert(folder.clone(), updated);
        // Return value of closure (which is *not* a return statement ;-)
        map
    });
}

fn write(to_file: &PathBuf, to_store: HashMap<String, Vec<Feed>>) {
    //    warn!("exporting feeds {:?}", to_store);
    // Prepare the document by setting all boilerplate elements (root, head, body, ...)
    let mut root = Element::new("opml");
    root.attributes
        .insert("version".to_owned(), "1.0".to_owned());
    let mut header = Element::new("head");
    let mut title = Element::new("title");
    title.text = Some("rrss2imap OPML Export".to_owned());
    header.children.push(title);
    root.children.push(header);
    let mut body = Element::new("body");
    // Now fill body with outline elements generated from feeds
    for (folder, elements) in to_store {
        let mut folder_element = Element::new("outline");
        folder_element
            .attributes
            .insert("text".to_owned(), folder.clone());
        folder_element
            .attributes
            .insert("title".to_owned(), folder.clone());
        for feed in elements {
            let mut outline = Element::new("outline");
            outline
                .attributes
                .insert("type".to_owned(), "rss".to_owned());
            outline
                .attributes
                .insert("text".to_owned(), feed.url.clone());
            outline
                .attributes
                .insert("xmlUrl".to_owned(), feed.url.clone());
            folder_element.children.push(outline);
        }
        body.children.push(folder_element);
    }
    // Don't forget to add body after, otherwise we enter into the dangerous realm of borrowed values
    root.children.push(body);
    let mut document = Document::new();
    document.root = Some(root);
    fs::write(to_file, format!("{}", document))
        .expect(&format!("Unable to write file {:?}", to_file));
}
