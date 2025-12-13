use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Topic {
    Status,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum ClientMessage {
    #[serde(rename = "subscribe")]
    Subscribe { topic: Topic },

    #[serde(rename = "unsubscribe")]
    Unsubscribe { topic: Topic },
}
