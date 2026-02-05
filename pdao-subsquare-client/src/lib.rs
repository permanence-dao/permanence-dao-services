use chrono::Utc;
use num_ordinal::{Ordinal, O32};
use pdao_config::Config;
use pdao_types::governance::policy::PolicyEvaluation;
use pdao_types::governance::subsquare::{
    SubSquareCommentData, SubSquareCommentIndexerData, SubSquareCommentReplyData,
    SubSquareCommentReplyRequest, SubSquareCommentRequest, SubSquareCommentResponse,
    SubSquareReferendum, SubSquareReferendumList,
};
use pdao_types::substrate::chain::Chain;
use sp_core::crypto::{Ss58AddressFormat, Ss58Codec};
use sp_core::{sr25519, Pair};

#[allow(clippy::too_many_arguments)]
fn get_vote_content(
    chain: &Chain,
    voting_policy_version: &str,
    cid: &str,
    previous_vote_count: u32,
    evaluation: &PolicyEvaluation,
    description_lines: &[String],
    _has_coi: bool,
    feedback_summary: &str,
    delegation_address: &str,
) -> anyhow::Result<String> {
    let vote = if let Some(vote) = evaluation.simplify()? {
        if vote {
            "AYE"
        } else {
            "NAY"
        }
    } else {
        "ABSTAIN"
    };

    let coi_disclaimer = ""; /*if has_coi {
        "<br><br>**DISCLAIMER:** Our Decentralized Voices delegation voted to abstain on this referendum in accordance with our conflict of interest policy, [announced](https://x.com/PermanenceDAO/status/1905223487976783987) on March 27th, 2025."
    } else {
        ""
    };
     */

    let content = format!(
        r#"Dear Proposer,

Thank you for your proposal. Our **{}** vote on this proposal is **{}**. Below is the evaluation of our [voting policy {}](https://docs.permanence.io/voting_policy/voting_policy_{}.html) on this referendum:

{}

Below is a summary of our members' comments:

> {feedback_summary}

The full discussion can be found in our [internal voting](https://voting.opensquare.io/space/permanence/proposal/{cid}).

Please feel free to contact us through the links below for further discussion.{coi_disclaimer}

Kind regards,<br>Permanence DAO<br>💬 [Public Telegram](https://t.me/permanencedao)<br>🌐️ [Web](https://permanence.io)<br>🐦 [Twitter](https://twitter.com/permanencedao)<br>🗳️ [Delegate](https://{}.subsquare.io/user/{}/votes)"#,
        O32::from1(previous_vote_count + 1),
        vote,
        voting_policy_version,
        voting_policy_version,
        description_lines.join("\n"),
        chain.chain,
        delegation_address,
    );
    Ok(content)
}

pub struct SubSquareClient {
    config: Config,
    http_client: reqwest::Client,
}

impl SubSquareClient {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            config: config.clone(),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(
                    config.http.request_timeout_seconds,
                ))
                .build()?,
        })
    }

    pub async fn fetch_referendum(
        &self,
        chain: &Chain,
        index: u32,
    ) -> anyhow::Result<Option<SubSquareReferendum>> {
        let url = format!(
            "https://{}-api.subsquare.io/gov2/referendums/{index}?simple=false",
            chain.chain,
        );
        let response = self.http_client.get(url).send().await?;
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        let refererendum = response.json::<SubSquareReferendum>().await?;
        Ok(Some(refererendum))
    }

    pub async fn fetch_referenda(
        &self,
        chain: &Chain,
        page: u16,
        page_size: u16,
    ) -> anyhow::Result<SubSquareReferendumList> {
        let url = format!(
            "https://{}-api.subsquare.io/gov2/referendums?simple=false&page_size={page_size}&page={page}",
            chain.chain,
        );
        Ok(self
            .http_client
            .get(url)
            .send()
            .await?
            .json::<SubSquareReferendumList>()
            .await?)
    }

    fn get_address(&self, chain: &Chain) -> String {
        let pair = sr25519::Pair::from_string(&self.config.substrate.gov_proxy_seed_phrase, None)
            .expect("Invalid seed phrase");
        pair.public()
            .to_ss58check_with_version(Ss58AddressFormat::from(chain.ss58_prefix))
    }

    fn sign(&self, data: &[u8]) -> String {
        let pair = sr25519::Pair::from_string(&self.config.substrate.gov_proxy_seed_phrase, None)
            .expect("Invalid seed phrase");
        let signature = pair.sign(data);
        format!("0x{}", hex::encode(signature))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn post_comment(
        &self,
        chain: &Chain,
        referendum: &SubSquareReferendum,
        cid: &str,
        previous_vote_count: u32,
        evaluation: &PolicyEvaluation,
        description_lines: &[String],
        has_coi: bool,
        feedback_summary: &str,
    ) -> anyhow::Result<SubSquareCommentResponse> {
        let url = format!(
            "https://{}-api.subsquare.io/sima/referenda/{}/comments",
            chain.chain, referendum.referendum_index,
        );
        let delegation_address = match chain.chain.as_str() {
            "polkadot" => self.config.voter.polkadot_real_account_address.as_str(),
            _ => self.config.voter.kusama_real_account_address.as_str(),
        };
        let content = get_vote_content(
            chain,
            &self.config.voter.voting_policy_version,
            cid,
            previous_vote_count,
            evaluation,
            description_lines,
            has_coi,
            feedback_summary,
            delegation_address,
        )?;
        let request_data = SubSquareCommentData {
            action: "comment".to_string(),
            indexer: SubSquareCommentIndexerData {
                pallet: "referenda".to_string(),
                object: "referendumInfoFor".to_string(),
                proposed_height: referendum.extrinsic.block_number,
                id: referendum.referendum_index,
            },
            content,
            content_format: "subsquare_md".to_string(),
            timestamp: Utc::now().timestamp_millis() as u64,
        };
        let request_data_json = serde_json::to_string(&request_data)?;
        let signature_hex = self.sign(&request_data_json.into_bytes());
        let request = SubSquareCommentRequest {
            entity: request_data,
            address: self.get_address(chain),
            signature: signature_hex,
            signer_wallet: "polkadot-js".to_string(),
        };
        let response_result = self.http_client.post(url).json(&request).send().await;
        let response = match response_result {
            Ok(response) => response,
            Err(error) => {
                log::error!(
                    "Error while posting SubSquare comment vote #{}: {}",
                    previous_vote_count + 1,
                    error
                );
                return Err(error.into());
            }
        };
        let status_code = response.status();
        let response_text = response.text().await?;
        if !status_code.is_success() {
            let error_message = format!(
                "Error while posting SubSquare comment for vote #{}: {}",
                previous_vote_count + 1,
                response_text
            );
            log::error!("{error_message}");
            return Err(anyhow::Error::msg(error_message));
        }
        log::info!(
            "Posted SubSquare comment for {} referendum #{} vote #{}. Response: {response_text}",
            chain.token_ticker,
            referendum.referendum_index,
            previous_vote_count + 1,
        );

        Ok(serde_json::from_str(&response_text)?)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn post_comment_reply(
        &self,
        chain: &Chain,
        referendum: &SubSquareReferendum,
        cid: &str,
        comment_cid: &str,
        previous_vote_count: u32,
        evaluation: &PolicyEvaluation,
        description_lines: &[String],
        has_coi: bool,
        feedback_summary: &str,
    ) -> anyhow::Result<SubSquareCommentResponse> {
        let url = format!(
            "https://{}-api.subsquare.io/sima/referenda/{}/comments/{comment_cid}/replies",
            chain.chain, referendum.referendum_index,
        );
        let delegation_address = match chain.chain.as_str() {
            "polkadot" => self.config.voter.polkadot_real_account_address.as_str(),
            _ => self.config.voter.kusama_real_account_address.as_str(),
        };
        let content = get_vote_content(
            chain,
            &self.config.voter.voting_policy_version,
            cid,
            previous_vote_count,
            evaluation,
            description_lines,
            has_coi,
            feedback_summary,
            delegation_address,
        )?;
        let request_data = SubSquareCommentReplyData {
            action: "comment".to_string(),
            comment_cid: comment_cid.to_string(),
            content,
            content_format: "subsquare_md".to_string(),
            timestamp: Utc::now().timestamp_millis() as u64,
        };
        let request_data_json = serde_json::to_string(&request_data)?;
        let signature_hex = self.sign(&request_data_json.into_bytes());
        let request = SubSquareCommentReplyRequest {
            entity: request_data,
            address: self.get_address(chain),
            signature: signature_hex,
            signer_wallet: "polkadot-js".to_string(),
        };
        let response_result = self.http_client.post(url).json(&request).send().await;
        let response = match response_result {
            Ok(response) => response,
            Err(error) => {
                log::error!("Error while posting SubSquare comment reply: {error}");
                return Err(error.into());
            }
        };
        let status_code = response.status();
        let response_text = response.text().await?;
        if !status_code.is_success() {
            let error_message =
                format!("Error while posting SubSquare comment reply: {response_text}");
            log::error!("{error_message}");
            return Err(anyhow::Error::msg(error_message));
        }
        log::info!(
            "Posted SubSquare comment reply for {} referendum ${}. Response: {response_text}",
            chain.token_ticker,
            referendum.referendum_index,
        );

        Ok(serde_json::from_str(&response_text)?)
    }
}
