#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub email:Option<String>,
    pub folder:Option<String>
}
