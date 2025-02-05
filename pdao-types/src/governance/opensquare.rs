use crate::governance::track::Track;
use crate::substrate::account_id::AccountId;
use crate::substrate::chain::Chain;
use chrono::{Datelike, Days, NaiveDate, NaiveDateTime, Utc};
use enum_iterator::Sequence;
use pdao_config::Config;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSquareNetworkAsset {
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSquareNetwork {
    pub network: String,
    pub ss58_format: u16,
    pub assets: Vec<OpenSquareNetworkAsset>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSquareSnapshotHeight {
    pub polkadot: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSquareNetworksConfig {
    pub symbol: String,
    pub decimals: u8,
    pub networks: Vec<OpenSquareNetwork>,
    pub accessibility: String,
    pub whitelist: Vec<String>,
    pub strategies: Vec<String>,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSquareNewProposal {
    pub space: String,
    pub networks_config: OpenSquareNetworksConfig,
    pub title: String,
    pub content: String,
    pub content_type: String,
    pub choice_type: String,
    pub choices: Vec<String>,
    pub start_date: u64,
    pub end_date: u64,
    pub snapshot_heights: OpenSquareSnapshotHeight,
    pub real_proposer: Option<AccountId>,
    pub proposer_network: String,
    pub version: String,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Sequence, Deserialize, Serialize, PartialEq)]
pub enum OpenSquareVote {
    Aye,
    Nay,
    Abstain,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpenSquareReferendumVote {
    #[serde(rename = "_id")]
    pub id: String,
    pub cid: String,
    #[serde(rename = "proposal")]
    pub proposal_id: String,
    pub voter: AccountId,
    pub address: AccountId,
    pub choices: Vec<OpenSquareVote>,
    pub remark: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSquareReferendum {
    #[serde(rename = "_id")]
    pub id: String,
    pub space: String,
    pub post_uid: String,
    pub title: String,
    pub content: String,
    pub proposer: AccountId,
    pub address: AccountId,
    pub status: String,
}

impl OpenSquareNewProposal {
    pub fn new(
        chain: &Chain,
        block_height: u64,
        config: &Config,
        referendum_index: u32,
        track: Track,
        title: String,
        content: String,
    ) -> Self {
        let space_chain = Chain::polkadot();
        let now = Utc::now();
        let day = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap();
        let start_of_day = NaiveDateTime::from(day);
        let end_date = now.checked_add_days(Days::new(30)).unwrap();
        let end_day =
            NaiveDate::from_ymd_opt(end_date.year(), end_date.month(), end_date.day()).unwrap();
        Self {
            space: config.referendum_importer.opensquare_space.clone(),
            networks_config: OpenSquareNetworksConfig {
                symbol: space_chain.token_ticker.clone(),
                decimals: space_chain.token_decimals as u8,
                networks: vec![OpenSquareNetwork {
                    network: space_chain.chain.clone(),
                    ss58_format: space_chain.ss58_prefix,
                    assets: vec![OpenSquareNetworkAsset {
                        symbol: space_chain.token_ticker.clone(),
                        decimals: space_chain.token_decimals as u8,
                    }],
                }],
                accessibility: "whitelist".to_string(),
                whitelist: vec![
                    "1ZSPR3zNg5Po3obkhXTPR95DepNBzBZ3CyomHXGHK9Uvx6w".to_string(),
                    "1xzcLSwo7xBFkJYZiL4EHaqFpuPTkH641E3V43W4cuk1bX6".to_string(),
                    "12His7t3EJ38tjdBbivUzWQeaNCLKfMqtKp1Ed3xHMyCE9N3".to_string(),
                    "12s6UMSSfE2bNxtYrJc6eeuZ7UxQnRpUzaAh1gPQrGNFnE8h".to_string(),
                    "13EDmaUe89xXocPppFmuoAZaCsckaJy3deAyVyiykk1zKQbF".to_string(),
                    "14333MZvbGkcq5CZ8fYHZiFYwHNDaW3uiErDKMb7oqnupWXn".to_string(),
                    "14gMJV95zwxUsFEZDSC8mtBVifS6SypKJkfBKANkMsLZdeVb".to_string(),
                    "14Gn7SEmCgMX7Ukuppnw5TRjA7pao2HFpuJo39frB42tYLEh".to_string(),
                    "15fTH34bbKGMUjF1bLmTqxPYgpg481imThwhWcQfCyktyBzL".to_string(),
                    "167YoKNriVtP4Nxk9F9GRV7HTKu5VnxaRq1pKMANAnmmTY9F".to_string(),
                    "13znFMMjHyM2UvSewvaKMC2bLUcySRMzcM8BAMTzm1G2P5ju".to_string(),
                ],
                strategies: vec!["one-person-one-vote".to_string()],
                version: "4".to_string(),
            },
            title: format!(
                "[{}] {} #{} - {}",
                track.short_name(),
                chain.token_ticker.clone(),
                referendum_index,
                title,
            ),
            content,
            content_type: "markdown".to_string(),
            choice_type: "single".to_string(),
            choices: vec!["Aye".to_string(), "Nay".to_string(), "Abstain".to_string()],
            start_date: start_of_day.and_utc().timestamp_millis() as u64,
            end_date: NaiveDateTime::from(end_day).and_utc().timestamp_millis() as u64,
            snapshot_heights: OpenSquareSnapshotHeight {
                polkadot: block_height,
            },
            real_proposer: None,
            proposer_network: space_chain.chain.clone(),
            version: "5".to_string(),
            timestamp: (Utc::now().timestamp_millis() / 1000) as u64,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSquareNewProposalRequest {
    pub data: OpenSquareNewProposal,
    pub address: String,
    pub signature: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSquareNewProposalResponse {
    pub cid: String,
    pub post_uid: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSquareReferendumVotesResponse {
    pub items: Vec<OpenSquareReferendumVote>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSquareTerminateProposalRequestData {
    pub action: String,
    pub proposal_cid: String,
    #[serde(rename = "terminatorNetwork")]
    pub chain: String,
    pub version: String,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSquareTerminateProposalRequest {
    pub data: OpenSquareTerminateProposalRequestData,
    pub address: String,
    pub signature: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSquareTerminateProposalResponse {
    pub result: bool,
}
