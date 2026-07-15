use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RagQuery {
    pub query: String,
    #[serde(default)]
    pub context_ids: String,
}
