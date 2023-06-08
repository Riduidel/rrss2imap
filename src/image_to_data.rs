use super::settings::*;

use base64::engine::*;

use lol_html::{rewrite_str, element, RewriteStrSettings};
use lol_html::errors::*;
use tests_bin::unit_tests;

#[unit_tests("image_to_data.rs")]
pub fn transform(document: &String) -> Result<String, RewritingError> {

    let output = rewrite_str(document,
        RewriteStrSettings {
            element_content_handlers: vec![
        // Rewrite images having src where src doesn't start with data
        element!("img[src]", |el| {
            let src = el
                .get_attribute("src")
                .unwrap();

            if !src.starts_with("data") {
                // Now it's time to rewrite!
                // Now download image source and base64 encode it !
                debug!("reading image from {}", src);
                if let Ok(mut response) = reqwest::blocking::get(src) {
                    let mut image: Vec<u8> = vec![];
                    response.copy_to(&mut image).unwrap();
                    let image_bytes = image.as_slice();
                    let encoded = general_purpose::STANDARD_NO_PAD.encode(image_bytes);
                    let image_mime_type = tree_magic::from_u8(image_bytes);
                    let encoded_image = format!("data:{};base64,{}", image_mime_type, encoded);
                    el.set_attribute("src", &encoded_image).unwrap();
                }
            }

            Ok(())
        })
    ],
            ..RewriteStrSettings::default()
        });
    output
}
