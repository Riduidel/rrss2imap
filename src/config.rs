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
}