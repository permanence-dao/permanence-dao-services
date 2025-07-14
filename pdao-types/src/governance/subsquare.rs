use crate::governance::ReferendumStatus;
use crate::substrate::account_id::AccountId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareBlock {
    #[serde(rename = "blockHeight")]
    pub number: u64,
    #[serde(rename = "blockHash")]
    pub hash: String,
    #[serde(rename = "blockTime")]
    pub timestamp: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareAssetKind {
    pub chain: String,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub symbol: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareReferendumState {
    #[serde(rename = "name")]
    pub status: ReferendumStatus,
    #[serde(rename = "indexer")]
    pub block: SubSquareBlock,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareReferendumContentSummary {
    pub summary: Option<String>,
    pub model: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareReferendumLocalSpend {
    pub is_spend_local: bool,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub symbol: String,
    pub amount: String,
    pub beneficiary: AccountId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareReferendumNonLocalSpend {
    pub is_spend_local: bool,
    pub asset_kind: SubSquareAssetKind,
    pub amount: String,
    pub beneficiary: SubSquareBeneficiary,
}

#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(untagged)]
pub enum SubSquareReferendumSpend {
    NonLocalSpend(SubSquareReferendumNonLocalSpend),
    LocalSpend(SubSquareReferendumLocalSpend),
}

#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareTrackInfo {
    pub id: u16,
    pub name: String,
    pub original_name: Option<String>,
    pub max_deciding: u32,
    pub decision_deposit: String,
    pub prepare_period: u32,
    pub decision_period: u32,
    pub confirm_period: u32,
    pub min_enactment_period: u32,
}

#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareReferendumOnChainDecisionInfo {
    #[serde(rename = "since")]
    pub decision_start_block_number: Option<u64>,
    #[serde(rename = "confirming")]
    pub confirm_start_block_number: Option<u64>,
}

#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareReferendumOnChainInfo {
    #[serde(rename = "submitted")]
    pub submission_block_number: u64,
    #[serde(rename = "deciding")]
    pub decision_info: Option<SubSquareReferendumOnChainDecisionInfo>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareReferendumOnChainData {
    pub info: SubSquareReferendumOnChainInfo,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareReferendum {
    #[serde(rename = "_id")]
    pub id: String,
    pub referendum_index: u32,
    #[serde(rename = "indexer")]
    pub extrinsic: SubSquareExtrinsic,
    pub proposer: AccountId,
    pub onchain_data: SubSquareReferendumOnChainData,
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
    pub track_info: SubSquareTrackInfo,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareReferendumList {
    pub items: Vec<SubSquareReferendum>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct SubSquareCommentIndexerData {
    pub pallet: String,
    pub object: String,
    pub proposed_height: u64,
    pub id: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct SubSquareCommentData {
    pub action: String,
    pub indexer: SubSquareCommentIndexerData,
    pub content: String,
    #[serde(rename = "content_format")]
    pub content_format: String,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareCommentRequest {
    pub entity: SubSquareCommentData,
    pub address: String,
    pub signature: String,
    pub signer_wallet: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareCommentResponse {
    pub cid: String,
    pub index: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct SubSquareCommentReplyData {
    pub action: String,
    #[serde(rename = "cid")]
    pub comment_cid: String,
    pub content: String,
    #[serde(rename = "content_format")]
    pub content_format: String,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareCommentReplyRequest {
    pub entity: SubSquareCommentReplyData,
    pub address: String,
    pub signature: String,
    pub signer_wallet: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSquareCommentReplyResponse {
    pub cid: String,
    pub index: u32,
}
