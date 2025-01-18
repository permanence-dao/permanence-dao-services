use pdao_config::Config;
use pdao_types::governance::opensquare::{
    OpenSquareNewProposal, OpenSquareNewProposalRequest, OpenSquareNewProposalResponse,
    OpenSquareReferendum, OpenSquareReferendumVote, OpenSquareReferendumVotesResponse,
};
use pdao_types::governance::subsquare::SubSquareReferendum;
use pdao_types::governance::track::Track;
use pdao_types::substrate::chain::Chain;
use sp_core::crypto::{Ss58AddressFormat, Ss58Codec};
use sp_core::{sr25519, Pair};

fn ellipsize(input: &str, limit: usize) -> String {
    // If input is already within the limit, just return it
    if input.chars().count() <= limit {
        return input.to_string();
    }

    // Otherwise, take the first `limit` characters, then add "..."
    let truncated: String = input.chars().take(limit - 3).collect();
    format!("{truncated}...")
}

pub struct OpenSquareClient {
    config: Config,
    http_client: reqwest::Client,
}

impl OpenSquareClient {
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
        cid: &str,
    ) -> anyhow::Result<Option<OpenSquareReferendum>> {
        let url = format!("https://voting.opensquare.io/api/permanence/proposal/{cid}",);
        let response = self.http_client.get(url).send().await?;
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        let refererendum = response.json::<OpenSquareReferendum>().await?;
        Ok(Some(refererendum))
    }

    pub async fn fetch_referendum_votes(
        &self,
        cid: &str,
    ) -> anyhow::Result<Option<Vec<OpenSquareReferendumVote>>> {
        let url = format!("https://voting.opensquare.io/api/permanence/proposal/{cid}/votes",);
        let response = self.http_client.get(url).send().await?;
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        let votes = response
            .json::<OpenSquareReferendumVotesResponse>()
            .await?
            .items;
        Ok(Some(votes))
    }

    pub async fn create_opensquare_proposal(
        &self,
        chain: &Chain,
        block_height: u64,
        referendum: &SubSquareReferendum,
    ) -> anyhow::Result<OpenSquareNewProposalResponse> {
        log::info!(
            "Create OpenSquare proposal for {} referendum {}.",
            chain.token_ticker,
            referendum.referendum_index
        );
        let pair = sr25519::Pair::from_string(&self.config.substrate.gov_proxy_seed_phrase, None)
            .expect("Invalid seed phrase");
        let address = pair
            .public()
            .to_ss58check_with_version(Ss58AddressFormat::from(chain.ss58_prefix));

        let content = format!(
            "https://{}.subsquare.io/referenda/{}\n\n{}",
            chain.chain,
            referendum.referendum_index,
            if let Some(content_summary) = &referendum.content_summary {
                if let Some(summary) = &content_summary.summary {
                    summary.clone()
                } else {
                    referendum.content.clone().unwrap_or("N/A".to_string())
                }
            } else {
                referendum.content.clone().unwrap_or("N/A".to_string())
            }
        );
        let proposal = OpenSquareNewProposal::new(
            chain,
            block_height,
            &self.config,
            referendum.referendum_index,
            Track::from_id(referendum.track_id).unwrap(),
            ellipsize(&referendum.title.clone().unwrap_or("N/A".to_string()), 130),
            content,
        );
        let proposal_json = serde_json::to_string(&proposal)?;
        let signature = pair.sign(proposal_json.as_bytes());
        let signature_hex = format!("0x{}", hex::encode(signature));
        let request = OpenSquareNewProposalRequest {
            data: proposal,
            address: address.clone(),
            signature: signature_hex,
        };
        let response_result = self
            .http_client
            .post(format!(
                "https://voting.opensquare.io/api/{}/proposals",
                self.config.referendum_importer.opensquare_space,
            ))
            .json(&request)
            .send()
            .await;
        let response = match response_result {
            Ok(response) => response,
            Err(error) => {
                log::error!("Error while creating OpenSquare proposal: {}", error);
                return Err(error.into());
            }
        };
        let status_code = response.status();
        let response_text = response.text().await?;
        if !status_code.is_success() {
            log::error!("Error response from OpenSquare proposal: {}", response_text);
        }
        let response: OpenSquareNewProposalResponse = serde_json::from_str(&response_text)?;
        log::info!(
            "Created OpenSquare proposal for {} referendum ${} with CID {}.",
            chain.token_ticker,
            referendum.referendum_index,
            response.cid,
        );
        Ok(response)
    }
}
