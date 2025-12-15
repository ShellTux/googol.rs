use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UserAgent {
    user_agent: String,
}

impl UserAgent {
    pub fn new(user_agent: String) -> Self {
        Self { user_agent }
    }
}
