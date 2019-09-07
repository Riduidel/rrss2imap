use super::feed::*;
use super::settings::*;

use kuchiki::*;

use base64;
use reqwest;

pub fn transform(document: NodeRef, _feed: &Feed, _settings: &Settings) -> NodeRef {
    for node_ref in document.select("img").unwrap() {
        // note we unwrapped the inner node to have its attributes available
        let node = node_ref.as_node().as_element();
        if let Some(data) = node {
            let attributes = &mut data.attributes.borrow_mut();
            if let Some(src) = attributes.get("src") {
                // Now download image source and base64 encode it !
                debug!("reading image from {}", src);
                if let Ok(mut response) = reqwest::get(src) {
                    let mut image: Vec<u8> = vec![];
                    response.copy_to(&mut image).unwrap();
                    let image_bytes = image.as_slice();
                    let encoded = base64::encode(image_bytes);
                    let image_mime_type = tree_magic::from_u8(image_bytes);
                    attributes.insert(
                        "src",
                        format!("data:{};base64,{}", image_mime_type, encoded),
                    );
                }
            }
        }
    }
    document
}
