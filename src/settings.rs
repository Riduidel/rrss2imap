/// Store-level config
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    #[serde(skip_serializing_if = "Settings::is_false", default="Settings::default_false")]
    pub do_not_save:bool
}

impl Settings {
    pub fn is_false(value:&bool)->bool {
        return !value;
    }
    pub fn default_false()->bool {
        return false;
    }

    pub fn default()->Settings {
        return Settings {
            do_not_save: false
        }
    }
}