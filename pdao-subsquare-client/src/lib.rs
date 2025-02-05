use chrono::Utc;
use num2words::{Lang, Num2Words};
use num_ordinal::{Ordinal, O32};
use pdao_config::Config;
use pdao_types::governance::policy::VotingPolicy;
use pdao_types::governance::subsquare::{
    SubSquareCommentData, SubSquareCommentIndexerData, SubSquareCommentRequest,
    SubSquareCommentResponse, SubSquareReferendum, SubSquareReferendumList,
};
use pdao_types::governance::track::Track;
use pdao_types::substrate::chain::Chain;
use sp_core::crypto::{Ss58AddressFormat, Ss58Codec};
use sp_core::{sr25519, Pair};

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
            "https://{}.subsquare.io/api/gov2/referendums/{index}?simple=false",
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
            "https://{}.subsquare.io/api/gov2/referendums?simple=false&page_size={page_size}&page={page}",
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

    #[allow(clippy::too_many_arguments)]
    pub async fn post_comment(
        &self,
        chain: &Chain,
        referendum: &SubSquareReferendum,
        cid: &str,
        track: &Track,
        policy: &VotingPolicy,
        previous_vote_count: u32,
        vote_distribution: (u32, u32, u32),
        member_count: u32,
        vote: Option<bool>,
        feedback_summary: &str,
    ) -> anyhow::Result<SubSquareCommentResponse> {
        let url = format!(
            "https://{}.subsquare.io/api/sima/referenda/{}/comments",
            chain.chain, referendum.referendum_index,
        );
        let policy_summary = match track {
            Track::Root
            | Track::WhitelistedCaller
            | Track::WishForChange
            | Track::Treasurer
            | Track::FellowshipAdmin
            | Track::BigSpender
            | Track::StakingAdmin
            | Track::LeaseAdmin
            | Track::GeneralAdmin
            | Track::AuctionAdmin
            | Track::ReferendumCanceller
            | Track::ReferendumKiller => format!("{}% quorum", policy.quorum_percent),
            Track::SmallTipper | Track::BigTipper | Track::SmallSpender => format!(
                "{}% participation and simple majority of all voters",
                policy.participation_percent
            ),
            Track::MediumSpender => {
                format!(
                    "{}% quorum and simple majority of all voters",
                    policy.quorum_percent
                )
            }
        };
        let abstain_summary = if vote_distribution.2 > 0 {
            format!(
                ", with **{} member{} abstaining**",
                Num2Words::new(vote_distribution.2)
                    .lang(Lang::English)
                    .to_words()
                    .unwrap(),
                if vote_distribution.2 > 1 { "s" } else { "" }
            )
        } else {
            "".to_string()
        };
        let content = format!(
            r#"Dear Proposer,

Thank you for your proposal. Our {} vote on this proposal is **{}**.

The **{}** track requires **{policy_summary}** according to our voting policy. This proposal has received **{} aye and {} nay** votes from **{} members**{abstain_summary}. Below is a summary of our members' comments:

> {feedback_summary}

The full discussion can be found in our [internal voting](https://voting.opensquare.io/space/permanence/proposal/{cid}).

Kind regards,<br>Permanence DAO"#,
            O32::from1(previous_vote_count + 1),
            if let Some(vote) = vote {
                if vote {
                    "AYE"
                } else {
                    "NAY"
                }
            } else {
                "ABSTAIN"
            },
            track.name(),
            Num2Words::new(vote_distribution.0)
                .lang(Lang::English)
                .to_words()
                .unwrap(),
            Num2Words::new(vote_distribution.1)
                .lang(Lang::English)
                .to_words()
                .unwrap(),
            Num2Words::new(member_count)
                .lang(Lang::English)
                .to_words()
                .unwrap(),
        );
        let pair = sr25519::Pair::from_string(&self.config.substrate.gov_proxy_seed_phrase, None)
            .expect("Invalid seed phrase");
        let address = pair
            .public()
            .to_ss58check_with_version(Ss58AddressFormat::from(chain.ss58_prefix));
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
        let signature = pair.sign(request_data_json.as_bytes());
        let signature_hex = format!("0x{}", hex::encode(signature));
        let request = SubSquareCommentRequest {
            entity: request_data,
            address,
            signature: signature_hex,
            signer_wallet: "polkadot-js".to_string(),
        };
        let response_result = self.http_client.post(url).json(&request).send().await;
        let response = match response_result {
            Ok(response) => response,
            Err(error) => {
                log::error!("Error while posting SubSquare comment: {}", error);
                return Err(error.into());
            }
        };
        let status_code = response.status();
        let response_text = response.text().await?;
        if !status_code.is_success() {
            let error_message =
                format!("Error while posting SubSquare proposal: {}", response_text);
            log::error!("{error_message}");
            return Err(anyhow::Error::msg(error_message));
        }
        log::info!(
            "Posted SubSquare comment for {} referendum ${}. Response: {response_text}",
            chain.token_ticker,
            referendum.referendum_index,
        );

        Ok(serde_json::from_str(&response_text)?)
    }
}
