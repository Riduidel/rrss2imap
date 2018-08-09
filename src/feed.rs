use super::config::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Feed {
    pub url:String,
    #[serde(skip_serializing_if = "Config::is_none")]
    pub config:Config
}

impl Feed {
    // Convert the parameters vec into a valid feed (if possible)
    pub fn from(parameters:Vec<String>) -> Feed {
        let mut consumed = parameters.clone();
        let url:String = consumed.pop().unwrap();
        let mut email:Option<String> = None;
        let mut folder:Option<String> = None;
        // If there is a second parameter, it can be either email or folder
        if !consumed.is_empty() {
            let second = consumed.pop().unwrap();
            // If second parameters contains an @, I suppose it is an email address
            if second.contains("@") {
                debug!("Second add parameter {} is considered an email address", second);
                email = Some(second)
            } else {
                warn!("Second add parameter {} is NOT considered an email address, but a folder. NO MORE ARGUMENTS WILL BE PROCESSED", second);
                folder = Some(second)
            }
        }
        // If there is a third parameter, it is the folder.
        // But if folder was already defined, there is an error !
        if !consumed.is_empty() && folder==None {
            folder = Some(consumed.pop().unwrap());
        }
        return Feed {
            url: url,
            config:Config {
                email: email,
                folder: folder
            }
        }
    }

    pub fn to_string(&self, config:&Config) -> String {
        return format!("{} {}", self.url, self.config.clone().to_string(config));
    }
}