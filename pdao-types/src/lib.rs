#![warn(clippy::disallowed_types)]

use crate::substrate::account_id::AccountId;
use std::fmt::{Display, Formatter};

pub mod err;
pub mod governance;
pub mod openai;
pub mod substrate;

#[derive(Clone, Debug, PartialEq)]
pub enum MembershipType {
    Core,
    Community,
}

impl Display for MembershipType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Core => "Core",
            Self::Community => "Community",
        };
        write!(f, "{str}")
    }
}

impl From<&str> for MembershipType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "core" => Self::Core,
            "community" => Self::Community,
            _ => panic!("Unkown membership type: {s}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Member {
    pub id: u32,
    pub name: String,
    pub telegram_username: String,
    pub polkadot_address: AccountId,
    pub polkadot_payment_address: AccountId,
    pub kusama_address: AccountId,
    pub kusama_payment_address: AccountId,
    pub is_on_leave: bool,
    pub membership_type: MembershipType,
}
