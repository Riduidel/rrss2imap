use std::path::PathBuf;

use std::fs::File;
use std::io::Read;

use super::feed::Feed;
use super::config::Config;
use super::store::Store;

use treexml::*;

pub fn import(from_file:&PathBuf, to_store:&mut Store) {
    let mut file = File::open(from_file).expect(&format!("Unable to open file {:?}", from_file));
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect(&format!("Unable to read file {:?}", from_file));

    let doc = Document::parse(contents.as_bytes()).unwrap();
    let root = doc.root.unwrap();

    // old style parsing is good, because it is old :-)
    for element in root.children {
        match element.name.as_ref() {
            "head" => debug!("Reading {}", element),
            "body" => import_body(element, to_store, &"".to_owned()),
            _ => error!("element {:?} was unexpected, please fill a bug !", element)
        }
    }
}

fn import_body(body:Element, to_store:&mut Store, folder:&String) {
    for element in body.children {
        match element.name.as_ref() {
            "outline" => import_outline(element, to_store, folder),
            _ => error!("element {:?} was unexpected, please fill a bug!", element)
        }
    }
}

fn import_outline(outline:Element, to_store:&mut Store, folder:&String) {
    if outline.children.is_empty() {
        // An outline without children is considered an OPML entry. Does it have the right set of attributes ?
        if outline.attributes.contains_key("type") && outline.attributes.contains_key("text") && outline.attributes.contains_key("xmlUrl") {
            let url = outline.attributes.get("xmlUrl");
            let feed = Feed {
                url: url.unwrap().to_string(),
                config: Config {
                    email: None,
                    folder: Some(folder.to_string())
                }
            };
            to_store.add_feed(feed);
        } else {
            error!("outline {:?} has no children, but doesn't has the right set of attributes. Please fill a bug!", outline.attributes);
        }
    } else {
        // An outline with children is considered an OPML folder. Does it have the right set of attributes ?
        if outline.attributes.contains_key("text") && outline.attributes.contains_key("title") {
            let folder = outline.attributes.get("text").unwrap();
            import_body(outline.clone(), to_store, &folder.to_string());
        } else {
            error!("outline {:?} has children, but doesn't has the right set of attributes. Please fill a bug!", outline.attributes);
        }
    }
}