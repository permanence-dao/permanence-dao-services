use pdao_config::Config;
use pdao_types::governance::opensquare::{NewProposalRequest, NewProposalResponse, Proposal};
use pdao_types::governance::subsquare::SubSquareReferendum;
use pdao_types::governance::track::Track;
use pdao_types::substrate::chain::Chain;
use sp_core::crypto::{Ss58AddressFormat, Ss58Codec};
use sp_core::{sr25519, Pair};

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

    pub async fn create_opensquare_proposal(
        &self,
        chain: &Chain,
        block_height: u64,
        referendum: &SubSquareReferendum,
    ) -> anyhow::Result<NewProposalResponse> {
        log::info!(
            "Create OpenSquare proposal for {} referendum ${}.",
            chain.token_ticker,
            referendum.referendum_index
        );
        let pair = sr25519::Pair::from_string(&self.config.substrate.gov_proxy_seed_phrase, None)
            .expect("Invalid seed phrase");
        let address = pair
            .public()
            .to_ss58check_with_version(Ss58AddressFormat::from(chain.ss58_prefix));

        let content = format!(
            "https://polkadot.subsquare.io/referenda/{}\n\n{}",
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
        let proposal = Proposal::new(
            chain,
            block_height,
            &self.config,
            referendum.referendum_index,
            Track::from_id(referendum.track_id).unwrap(),
            referendum.title.clone().unwrap_or("N/A".to_string()),
            content,
        );
        let proposal_json = serde_json::to_string(&proposal)?;
        let signature = pair.sign(proposal_json.as_bytes());
        let signature_hex = format!("0x{}", hex::encode(signature));
        let request = NewProposalRequest {
            data: proposal,
            address: address.clone(),
            signature: signature_hex,
        };
        let response = self
            .http_client
            .post(format!(
                "https://voting.opensquare.io/api/{}/proposals",
                self.config.referendum_importer.opensquare_space,
            ))
            .json(&request)
            .send()
            .await?;
        let response: NewProposalResponse = response.json().await?;
        log::info!(
            "Created OpenSquare proposal for {} referendum ${} with CID {}.",
            chain.token_ticker,
            referendum.referendum_index,
            response.cid,
        );
        Ok(response)
    }
}
