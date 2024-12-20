use crate::governance::ReferendumStatus;
use crate::substrate::account_id::AccountId;
use frame_support::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareBlock {
    #[serde(rename = "blockHeight")]
    pub number: u64,
    #[serde(rename = "blockHash")]
    pub hash: String,
    #[serde(rename = "blockTime")]
    pub timestamp: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareExtrinsic {
    #[serde(rename = "blockHeight")]
    pub block_number: u64,
    pub block_hash: String,
    #[serde(rename = "blockTime")]
    pub block_timestamp: u64,
    pub extrinsic_index: u32,
    pub event_index: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareAssetKind {
    pub chain: String,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub symbol: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareBeneficiary {
    pub chain: String,
    pub address: String,
    #[serde(rename = "pubKey")]
    pub public_key: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareReferendumSummary {
    pub summary: String,
    #[serde(rename = "indexer")]
    pub block: SubSquareBlock,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareReferendumState {
    #[serde(rename = "name")]
    pub status: ReferendumStatus,
    #[serde(rename = "indexer")]
    pub block: SubSquareBlock,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareReferendumContentSummary {
    pub summary: Option<String>,
    pub model: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareReferendumLocalSpend {
    pub is_spend_local: bool,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub symbol: String,
    pub amount: String,
    pub beneficiary: AccountId,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareReferendumNonLocalSpend {
    pub is_spend_local: bool,
    pub asset_kind: SubSquareAssetKind,
    pub amount: String,
    pub beneficiary: SubSquareBeneficiary,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum SubSquareReferendumSpend {
    NonLocalSpend(SubSquareReferendumNonLocalSpend),
    LocalSpend(SubSquareReferendumLocalSpend),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareReferendum {
    #[serde(rename = "_id")]
    pub id: String,
    pub referendum_index: u32,
    #[serde(rename = "indexer")]
    pub extrinsic: SubSquareExtrinsic,
    pub proposer: AccountId,
    pub title: Option<String>,
    pub content: Option<String>,
    pub content_type: String,
    #[serde(rename = "track")]
    pub track_id: u16,
    pub state: SubSquareReferendumState,
    #[serde(rename = "edited")]
    pub is_edited: Option<bool>,
    pub content_summary: Option<SubSquareReferendumContentSummary>,
    pub all_spends: Option<Vec<SubSquareReferendumSpend>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareReferendumList {
    pub items: Vec<SubSquareReferendum>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}
