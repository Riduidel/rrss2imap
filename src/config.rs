///
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
    pub fn to_string(self, default:&Config) -> String {
        return format!("(to: {}) {}",
            self.email.unwrap_or(format!("{} (default)", default.clone().email.unwrap_or("".to_owned()))),
            self.folder.unwrap_or(format!("{} (default)", default.clone().folder.unwrap_or("".to_owned())))
        );
    }

    pub fn is_none(config:&Config) -> bool {
        return config.email.is_none() && config.folder.is_none();
    }
}
