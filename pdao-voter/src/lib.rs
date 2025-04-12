use pdao_config::Config;
use pdao_types::substrate::chain::Chain;

mod remove_vote;
mod vote;

#[subxt::subxt(runtime_metadata_path = "../_metadata/polkadot-metadata.scale")]
mod polkadot {}

#[subxt::subxt(runtime_metadata_path = "../_metadata/kusama-metadata.scale")]
mod kusama {}

pub struct Voter {
    config: Config,
}

impl Voter {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    pub async fn remove_vote(
        &self,
        chain: &Chain,
        referendum_index: u32,
    ) -> anyhow::Result<(String, u64, u32)> {
        match chain.chain.as_str() {
            "polkadot" => self.remove_vote_polkadot(chain, referendum_index).await,
            _ => self.remove_vote_kusama(chain, referendum_index).await,
        }
    }

    pub async fn vote(
        &self,
        chain: &Chain,
        referendum_index: u32,
        has_coi: bool,
        vote: Option<bool>,
        balance: u128,
        conviction: u8,
    ) -> anyhow::Result<(String, u64, u32)> {
        match chain.chain.as_str() {
            "polkadot" => {
                self.vote_polkadot(chain, referendum_index, has_coi, vote, balance, conviction)
                    .await
            }
            _ => {
                self.vote_kusama(chain, referendum_index, has_coi, vote, balance, conviction)
                    .await
            }
        }
    }
}
