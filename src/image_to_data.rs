

use base64::engine::*;
use microkv::MicroKV;
use std::sync::Mutex;

use lol_html::{rewrite_str, element, RewriteStrSettings};
use lol_html::errors::*;
use tests_bin::unit_tests;

#[unit_tests("image_to_data.rs")]
pub fn transform(document: &String) -> Result<String, RewritingError> {
    lazy_static! {
        // initialize in-memory database with (unsafe) cleartext password
        static ref DB: Mutex<MicroKV> = Mutex::new(MicroKV::new("my_db")
            // I don't care about the password, since I only maintain an in-memory cache
            .with_pwd_clear("my_password_123"));
    }

    
    rewrite_str(document,
        RewriteStrSettings {
            element_content_handlers: vec![
        // Rewrite images having src where src doesn't start with data
        element!("img[src]", |el| {
            let src:String = el
                .get_attribute("src")
                .unwrap();

            let db = DB.lock().unwrap();
            if !src.starts_with("data") {
                let db_content = db.get_unwrap::<String>(&src);
                match db_content {
                    Ok(base64) => el.set_attribute("src", &base64).unwrap(),
                    Err(_) => {
                        // Now it's time to rewrite!
                        // Now download image source and base64 encode it !
                        debug!("reading image from {}", &src);
                        if let Ok(mut response) = reqwest::blocking::get(&src) {
                            let mut image: Vec<u8> = vec![];
                            response.copy_to(&mut image).unwrap();
                            let image_bytes = image.as_slice();
                            let encoded = general_purpose::STANDARD_NO_PAD.encode(image_bytes);
                            let image_mime_type = tree_magic::from_u8(image_bytes);
                            let encoded_image = format!("data:{};base64,{}", image_mime_type, encoded);
                            el.set_attribute("src", &encoded_image).unwrap();
                            // Yeah we really don't care abut the fact that the value is inserted or not in cache, because
                            // it's a cache
                            db.put(&src, &encoded_image);
                        }
                    }
                }
            }

            Ok(())
        })
    ],
            ..RewriteStrSettings::default()
        })
}
