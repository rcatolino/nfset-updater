use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Prefix {
    #[serde(rename = "Prefix")]
    pub prefix: String,
    #[serde(rename = "Count")]
    pub count: i32,
    #[serde(rename = "Total")]
    pub total: i32,
}

#[derive(Deserialize)]
pub struct Response {
    pub prefixes: Vec<Prefix>,
}
