use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SearchBody {
    pub words: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub query: String,
}
