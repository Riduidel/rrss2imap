/// This structure defines the feed-level config.
/// All elements here may be configured twice : once at feed level, and once at global level.
/// Obviously, all elements which are not defined at feed level use global configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email:Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder:Option<String>
}

impl Config {
    pub fn new() -> Config {
        return Config {
            email: None,
            folder: None
        }
    }

    pub fn to_string(self, default:&Config) -> String {
        return format!("(to: {}) {}",
            self.email.unwrap_or(format!("{} (default)", default.clone().email.unwrap_or("".to_owned()))),
            self.folder.unwrap_or(format!("{} (default)", default.clone().folder.unwrap_or("".to_owned())))
        );
    }

    /// Used by serde to skip serialization of default config for feeds
    /// This method check if config is the default one (consisting only into None options)
    pub fn is_none(config:&Config) -> bool {
        return config.email.is_none() && config.folder.is_none();
    }

    /// Clear all content from this config excepted email address
    pub fn clear(&mut self) {
        self.folder = None;
    }

    pub fn get_email(self, default:&Config) -> String {
        return self.email.unwrap_or(default.clone().email.unwrap_or("".to_owned()));
    }

    pub fn get_folder(self, default:&Config) -> String {
        return self.folder.unwrap_or(default.clone().folder.unwrap_or("".to_owned()));
    }
}
