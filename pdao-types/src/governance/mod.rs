use crate::governance::track::Track;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub mod opensquare;
pub mod policy;
pub mod subsquare;
pub mod track;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum ReferendumStatus {
    Confirming,
    Deciding,
    Queueing,
    Preparing,
    Submitted,
    Approved,
    Cancelled,
    Killed,
    TimedOut,
    Rejected,
    Executed,
}

impl ReferendumStatus {
    pub fn get_icon(&self) -> &str {
        match self {
            ReferendumStatus::Confirming => "🟢",
            ReferendumStatus::Deciding => "🗳️",
            ReferendumStatus::Queueing => "📥",
            ReferendumStatus::Preparing => "🛠️",
            ReferendumStatus::Submitted => "📨",
            ReferendumStatus::Approved => "✅",
            ReferendumStatus::Cancelled => "🚫",
            ReferendumStatus::Killed => "💀",
            ReferendumStatus::TimedOut => "⌛",
            ReferendumStatus::Rejected => "❌",
            ReferendumStatus::Executed => "🎯",
        }
    }

    pub fn requires_termination(&self) -> bool {
        match self {
            ReferendumStatus::Confirming => false,
            ReferendumStatus::Deciding => false,
            ReferendumStatus::Queueing => false,
            ReferendumStatus::Preparing => false,
            ReferendumStatus::Submitted => false,
            ReferendumStatus::Approved => true,
            ReferendumStatus::Cancelled => true,
            ReferendumStatus::Killed => true,
            ReferendumStatus::TimedOut => true,
            ReferendumStatus::Rejected => true,
            ReferendumStatus::Executed => true,
        }
    }
}

impl Display for ReferendumStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            ReferendumStatus::Confirming => "Confirming",
            ReferendumStatus::Deciding => "Deciding",
            ReferendumStatus::Queueing => "Queueing",
            ReferendumStatus::Preparing => "Preparing",
            ReferendumStatus::Submitted => "Submitted",
            ReferendumStatus::Approved => "Approved",
            ReferendumStatus::Cancelled => "Cancelled",
            ReferendumStatus::Killed => "Killed",
            ReferendumStatus::TimedOut => "TimedOut",
            ReferendumStatus::Rejected => "Rejected",
            ReferendumStatus::Executed => "Executed",
        };
        write!(f, "{display}")
    }
}

impl FromStr for ReferendumStatus {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Confirming" => Ok(ReferendumStatus::Confirming),
            "Deciding" => Ok(ReferendumStatus::Deciding),
            "Queueing" => Ok(ReferendumStatus::Queueing),
            "Preparing" => Ok(ReferendumStatus::Preparing),
            "Submitted" => Ok(ReferendumStatus::Submitted),
            "Approved" => Ok(ReferendumStatus::Approved),
            "Cancelled" => Ok(ReferendumStatus::Cancelled),
            "Killed" => Ok(ReferendumStatus::Killed),
            "TimedOut" => Ok(ReferendumStatus::TimedOut),
            "Rejected" => Ok(ReferendumStatus::Rejected),
            "Executed" => Ok(ReferendumStatus::Executed),
            _ => panic!("Unknown referendum status: {s}"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Referendum {
    pub id: u32,
    pub network_id: u32,
    pub track: Track,
    pub index: u32,
    pub status: ReferendumStatus,
    pub title: Option<String>,
    pub content: Option<String>,
    pub content_type: String,
    pub telegram_chat_id: i64,
    pub telegram_topic_id: i32,
    pub telegram_intro_message_id: i32,
    pub opensquare_cid: Option<String>,
    pub opensquare_post_uid: Option<String>,
    pub last_vote_id: Option<u32>,
    pub is_terminated: bool,
    pub has_coi: bool,
    pub is_archived: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vote {
    pub id: u32,
    pub network_id: u32,
    pub referendum_id: u32,
    pub index: u32,
    pub block_hash: String,
    pub block_number: u64,
    pub extrinsic_index: u32,
    pub vote: Option<bool>,
    pub balance: u128,
    pub conviction: u32,
    pub is_removed: bool,
    pub subsquare_comment_cid: Option<String>,
    pub subsquare_comment_index: Option<u32>,
}
