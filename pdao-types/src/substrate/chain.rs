use sp_core::crypto::Ss58AddressFormat;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Chain {
    pub id: u32,
    pub chain: String,
    pub display: String,
    pub rpc_url: String,
    pub asset_hub_rpc_url: String,
    pub token_ticker: String,
    pub token_decimals: usize,
    pub token_format_decimal_points: usize,
    pub ss58_prefix: u16,
    pub block_time_seconds: u16,
}

impl Chain {
    pub fn polkadot() -> Self {
        Chain {
            id: 1,
            chain: "polkadot".to_string(),
            display: "Polkadot".to_string(),
            rpc_url: "wss://rpc.helikon.io:443/polkadot".to_string(),
            asset_hub_rpc_url: "wss://rpc.helikon.io:443/asset-hub-polkadot".to_string(),
            token_ticker: "DOT".to_string(),
            token_decimals: 10,
            token_format_decimal_points: 4,
            ss58_prefix: 0,
            block_time_seconds: 6,
        }
    }

    pub fn polkadot_asset_hub() -> Self {
        Chain {
            id: 2,
            chain: "polkadot asset hub".to_string(),
            display: "Polkadot Asset Hub".to_string(),
            rpc_url: "wss://rpc.helikon.io:443/asset-hub-polkadot".to_string(),
            asset_hub_rpc_url: "wss://rpc.helikon.io:443/asset-hub-polkadot".to_string(),
            token_ticker: "KSM".to_string(),
            token_decimals: 12,
            token_format_decimal_points: 4,
            ss58_prefix: 2,
            block_time_seconds: 6,
        }
    }

    pub fn kusama() -> Self {
        Chain {
            id: 2,
            chain: "kusama".to_string(),
            display: "Kusama".to_string(),
            rpc_url: "wss://rpc.helikon.io:443/kusama".to_string(),
            asset_hub_rpc_url: "wss://rpc.helikon.io:443/asset-hub-kusama".to_string(),
            token_ticker: "KSM".to_string(),
            token_decimals: 12,
            token_format_decimal_points: 4,
            ss58_prefix: 2,
            block_time_seconds: 6,
        }
    }

    pub fn kusama_asset_hub() -> Self {
        Chain {
            id: 2,
            chain: "kusama asset hub".to_string(),
            display: "Kusama Asset Hub".to_string(),
            rpc_url: "wss://rpc.helikon.io:443/asset-hub-kusama".to_string(),
            asset_hub_rpc_url: "wss://rpc.helikon.io:443/asset-hub-kusama".to_string(),
            token_ticker: "KSM".to_string(),
            token_decimals: 12,
            token_format_decimal_points: 4,
            ss58_prefix: 2,
            block_time_seconds: 6,
        }
    }
}

impl Display for Chain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display)
    }
}

#[derive(Debug)]
pub struct ParseChainError(String);

impl Display for ParseChainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseChainError {}

impl FromStr for Chain {
    type Err = ParseChainError;

    /// Get chain from string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "kusama" | "ksm" => Ok(Self::kusama()),
            "kusama asset hub" | "kah" => Ok(Self::kusama_asset_hub()),
            "polkadot" | "dot" => Ok(Self::polkadot()),
            "polkadot asset hub" | "pah" => Ok(Self::polkadot_asset_hub()),
            _ => Err(ParseChainError(format!("Unknown chain: {s}"))),
        }
    }
}

impl Chain {
    pub fn from_id(id: u32) -> Self {
        match id {
            2 => Self::kusama(),
            _ => Self::polkadot(),
        }
    }

    pub fn sp_core_set_default_ss58_version(&self) {
        sp_core::crypto::set_default_ss58_version(Ss58AddressFormat::from(self.ss58_prefix));
    }
}
