use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EnqueueInput {
    pub url: String,
}
