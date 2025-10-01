use pdao_config::{Config, OpenAPIConfig};
use pdao_types::governance::opensquare::{OpenSquareReferendumVote, OpenSquareVote};
use pdao_types::governance::policy::VotingPolicyEvaluation;
use pdao_types::governance::subsquare::SubSquareReferendum;
use pdao_types::openai::{
    OpenAICompletionRequest, OpenAICompletionResponse, OpenAIMessage, OpenAIModel, OpenAIRole,
};
use pdao_types::substrate::chain::Chain;

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
        chain: &Chain,
        sub_square_referendum: &SubSquareReferendum,
        vote: VotingPolicyEvaluation,
        votes: &[OpenSquareReferendumVote],
    ) -> anyhow::Result<String> {
        let mut prompt_parts: Vec<String> = Vec::new();
        prompt_parts.push("You are a chatbot specializing in Polkadot OpenGov, helping the members of Permanence DAO by preparing summaries of their comments for their votes on certain referenda.".to_owned());
        prompt_parts.push(format!(
            "This specific job is for {} referendum number {}, titled '{}'",
            chain.display,
            sub_square_referendum.referendum_index,
            serde_json::to_string(&sub_square_referendum.title)?
        ));
        prompt_parts.push(
            "Below is the detailed information for the referendum in JSON format.".to_owned(),
        );
        let json = serde_json::to_string(&sub_square_referendum)?;
        prompt_parts.push(serde_json::to_string(&json)?);
        let vote = match vote {
            VotingPolicyEvaluation::AbstainThresholdNotMet { .. } => "ABSTAIN",
            VotingPolicyEvaluation::ParticipationNotMet { .. } => "NO VOTE",
            VotingPolicyEvaluation::AyeAbstainMajorityAbstain { .. } => "ABSTAIN",
            VotingPolicyEvaluation::MajorityAbstain { .. } => "ABSTAIN",
            VotingPolicyEvaluation::AyeEqualsNayAbstain { .. } => "ABSTAIN",
            VotingPolicyEvaluation::Aye { .. } => "AYE",
            VotingPolicyEvaluation::Nay { .. } => "NAY",
        };
        prompt_parts.push(format!(
            "The vote of the DAO on this referendum determined by its voting policy that applies to this referendum is {vote}.",
        ));
        prompt_parts.push("Below the list of voters, their votes, and their comments for their votes. If the voter did not post comment for their vote, it is noted as 'No comment.'.".to_owned());
        prompt_parts.push("Summarize all these in a single paragraph of roughly 100 words, without referring to the names of the voters and their specific votes.".to_owned());
        prompt_parts.push("Use past tense. Do not start the summary with generic terms such as 'In a recent referendum...' etc. Do not start the summary with a generic sentence such as 'The members of the DAO...'.".to_owned());
        for (i, vote) in votes.iter().enumerate() {
            let mut vote_prompt = format!("\n\nVoter No.{i}");
            if vote.choices.contains(&OpenSquareVote::Aye) {
                vote_prompt = format!("{vote_prompt} :: Aye");
            } else if vote.choices.contains(&OpenSquareVote::Nay) {
                vote_prompt = format!("{vote_prompt} :: Nay");
            } else if vote.choices.contains(&OpenSquareVote::Abstain) {
                vote_prompt = format!("{vote_prompt} :: Abstain");
            }
            if !vote.remark.is_empty() {
                vote_prompt = format!("{vote_prompt}\n{}", serde_json::to_string(&vote.remark)?);
            } else {
                vote_prompt = "No comment.".to_owned();
            }
            prompt_parts.push(vote_prompt);
        }
        let prompt = prompt_parts.join("\n\n");
        let request = OpenAICompletionRequest {
            model: OpenAIModel::O3Mini20250131,
            messages: vec![OpenAIMessage {
                role: OpenAIRole::User,
                content: prompt,
            }],
            store: true,
            temperature: 1.0,
        };
        self.fetch_response(request).await
    }
}
