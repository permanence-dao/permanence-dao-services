use pdao_config::{Config, OpenAPIConfig};
use pdao_types::governance::opensquare::{OpenSquareReferendumVote, OpenSquareVote};
use pdao_types::openai::{
    OpenAICompletionRequest, OpenAICompletionResponse, OpenAIMessage, OpenAIModel, OpenAIRole,
};

pub struct OpenAIClient {
    config: OpenAPIConfig,
    http_client: reqwest::Client,
}

impl OpenAIClient {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(
                    config.http.request_timeout_seconds,
                ))
                .build()?,
            config: config.openai.clone(),
        })
    }

    async fn fetch_response(&self, request: OpenAICompletionRequest) -> anyhow::Result<String> {
        let response_result = self
            .http_client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("OpenAI-Project", &self.config.project)
            .header("OpenAI-Organization", &self.config.organization)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await;
        let response = match response_result {
            Ok(response) => response,
            Err(error) => {
                log::error!("OpenAI request error: {error}");
                return Err(error.into());
            }
        };
        let status_code = response.status();
        let response_text = response.text().await?;
        if !status_code.is_success() {
            let error_message = format!("OpenAI request error: {response_text}");
            log::error!("{error_message}");
            return Err(anyhow::Error::msg(error_message));
        }
        let response: OpenAICompletionResponse = serde_json::from_str(&response_text)?;
        if let Some(response) = response.choices.first() {
            Ok(response.message.content.clone())
        } else {
            let error_message = format!("Empty response from OpenAI: {response_text}");
            Err(anyhow::Error::msg(error_message))
        }
    }

    pub async fn fetch_chat_response(
        &self,
        sender_username: &str,
        message: &str,
    ) -> anyhow::Result<String> {
        let prompt = format!("Your name is Permie. You are a chatbot added to the Telegram chat group of Permanence DAO, a DAO that focuses on Open Governance in Polkadot. You specialize in Polkadot OpenGov, and assist the DAO members with their decision-making processes, and other issues related to blockchain governance. You received a message from user {sender_username}. Respond in no more than 200 words.");
        let request = OpenAICompletionRequest {
            model: OpenAIModel::GPT4OMini,
            messages: vec![
                OpenAIMessage {
                    role: OpenAIRole::Developer,
                    content: prompt,
                },
                OpenAIMessage {
                    role: OpenAIRole::User,
                    content: message.to_string(),
                },
            ],
            store: false,
            temperature: 0.5,
        };
        self.fetch_response(request).await
    }

    pub async fn fetch_feedback_summary(
        &self,
        votes: &[OpenSquareReferendumVote],
    ) -> anyhow::Result<String> {
        let mut prompt_parts: Vec<String> = Vec::new();
        prompt_parts.push("I'm giving you below a list of voters, their votes, and their comments on a referendum. Please summarize all these in a single paragraph of roughly 80 words, without referring to the names of the voters and their specific votes. Use past tense.".to_string());
        for (i, vote) in votes.iter().enumerate() {
            let mut vote_prompt = format!("Voter No.{i}");
            if vote.choices.contains(&OpenSquareVote::Aye) {
                vote_prompt = format!("{vote_prompt} :: Aye");
            } else if vote.choices.contains(&OpenSquareVote::Nay) {
                vote_prompt = format!("{vote_prompt} :: Nay");
            } else if vote.choices.contains(&OpenSquareVote::Abstain) {
                vote_prompt = format!("{vote_prompt} :: Abstain");
            }
            if !vote.remark.is_empty() {
                vote_prompt = format!("{vote_prompt}\n{}", vote.remark);
            }
            prompt_parts.push(vote_prompt);
        }
        let prompt = prompt_parts.join("\n\n");
        let request = OpenAICompletionRequest {
            model: OpenAIModel::GPT4OMini,
            messages: vec![OpenAIMessage {
                role: OpenAIRole::User,
                content: prompt,
            }],
            store: false,
            temperature: 0.5,
        };
        self.fetch_response(request).await
    }
}
