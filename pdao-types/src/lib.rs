#![warn(clippy::disallowed_types)]
use crate::substrate::account_id::AccountId;

pub mod err;
pub mod governance;
pub mod openai;
pub mod substrate;

#[derive(Clone, Debug)]
pub struct Member {
    pub name: String,
    pub telegram_username: String,
    pub polkadot_address: AccountId,
    pub polkadot_payment_address: AccountId,
    pub kusama_address: AccountId,
    pub kusama_payment_address: AccountId,
    pub is_on_leave: bool,
}
