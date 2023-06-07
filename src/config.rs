use super::settings::*;

/// This structure defines the feed-level config.
/// All elements here may be configured twice : once at feed level, and once at global level.
/// Obviously, all elements which are not defined at feed level use global configuration
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Config {
    /// When set, contains the email address used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// When set, contains the folder in which entries for feed will be written
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    /// When defined, this from field will be used instead of trying to construct it from feed title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// When set to true, images will be inlined
    #[serde(
        skip_serializing_if = "Settings::is_false",
        default = "Settings::default_false"
    )]
    pub inline_image_as_data: bool,
}

impl Config {
    /// Creates a new instance with all fields set to default "falsy" values : options are set to none and booleans to false
    pub fn new() -> Config {
        Config {
            email: None,
            folder: None,
            inline_image_as_data: false,
            from: None,
        }
    }

    /// Creates a string view of config.
    /// More precisely, outputs the email address and folder in which entries are to be written
    /// A default config is given for options set to None.
    pub fn to_string(self, default: &Config) -> String {
        format!(
            "(to: {}) {}",
            self.email.unwrap_or_else(|| format!(
                "{} (default)",
                default.clone().email.unwrap_or_else(|| "".to_owned())
            )),
            self.folder.unwrap_or_else(|| format!(
                "{} (default)",
                default.clone().folder.unwrap_or_else(|| "".to_owned())
            ))
        )
    }

    /// Used by serde to skip serialization of default config for feeds
    /// This method check if config is the default one (consisting only into None options)
    pub fn is_none(config: &Config) -> bool {
        config.email.is_none()
            && config.folder.is_none()
            && config.from.is_none()
            && !config.inline_image_as_data
    }

    /// Clear all content from this config excepted email address
    pub fn clear(&mut self) {
        self.folder = None;
    }

    /// Get the email value for that feed, be it defined locally or from the default config
    pub fn get_email(&self, default: &Config) -> String {
        self.clone()
            .email
            .unwrap_or_else(|| default.clone().email.unwrap_or_else(|| "".to_owned()))
    }

    /// Get the folder value for that feed, be it defined locally or from the default config
    pub fn get_folder(&self, default: &Config) -> String {
        self.clone()
            .folder
            .unwrap_or_else(|| default.clone().folder.unwrap_or_else(|| "".to_owned()))
    }

    /// Compute an inline flag by resolving the two flags with this struct inline images status
    pub fn inline(&self, inline:bool, do_not_inline:bool)->bool {
        if self.inline_image_as_data {
            !do_not_inline
        } else {
            inline
        }
    }
}
