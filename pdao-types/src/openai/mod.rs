use frame_support::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum OpenAIRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "developer")]
    Developer,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum OpenAIModel {
    #[serde(rename = "gpt-4o-mini")]
    GPT4OMini,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpenAIMessage {
    pub role: OpenAIRole,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpenAICompletionRequest {
    pub model: OpenAIModel,
    pub messages: Vec<OpenAIMessage>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpenAICompletionChoice {
    pub index: u32,
    pub message: OpenAIMessage,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpenAICompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub choices: Vec<OpenAICompletionChoice>,
}
