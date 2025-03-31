use pdao_config::Config;
use pdao_substrate_client::SubstrateClient;
use pdao_types::substrate::chain::Chain;
use std::str::FromStr;
use subxt::utils::AccountId32;
use subxt::{OnlineClient, PolkadotConfig};
use subxt_signer::{sr25519, SecretUri};

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
    ) -> anyhow::Result<(u64, u32)> {
        use polkadot::proxy::calls::types::proxy::Call;
        use polkadot::runtime_types::pallet_conviction_voting::pallet::Call as ConvictionVotingCall;

        let api = OnlineClient::<PolkadotConfig>::from_url(&chain.rpc_url).await?;
        let call = ConvictionVotingCall::remove_vote {
            class: None,
            index: referendum_index,
        };
        let call = Call::ConvictionVoting(call);
        let real = AccountId32::from_str(&self.config.voter.polkadot_real_account_address)?;
        let proxy = polkadot::tx().proxy().proxy(real.into(), None, call);
        let uri = SecretUri::from_str(&self.config.voter.polkadot_proxy_account_seed_phrase)
            .expect("Invalid seed phrase.");
        let keypair = sr25519::Keypair::from_uri(&uri).expect("Invalid keypair.");
        let tx_progress = api
            .tx()
            .sign_and_submit_then_watch_default(&proxy, &keypair)
            .await?;
        let tx_in_block = tx_progress.wait_for_finalized().await?;
        let block_hash = tx_in_block.block_hash();
        let block_hash = format!("0x{}", hex::encode(block_hash.0));
        let events = tx_in_block.wait_for_success().await?;
        let subtrate_client = SubstrateClient::new(
            &chain.rpc_url,
            self.config.substrate.connection_timeout_seconds,
            self.config.substrate.request_timeout_seconds,
        )
        .await?;
        let header = subtrate_client.get_block_header(&block_hash).await?;
        let block_number = header.get_number()?;
        let extrinsic_index = events.extrinsic_index();
        Ok((block_number, extrinsic_index))
    }

    pub async fn vote(
        &self,
        chain: &Chain,
        referendum_index: u32,
        vote: Option<bool>,
        balance: u128,
        conviction: u8,
    ) -> anyhow::Result<(String, u64, u32)> {
        match chain.chain.as_str() {
            "polkadot" => {
                self.vote_polkadot(chain, referendum_index, vote, balance, conviction)
                    .await
            }
            _ => {
                self.vote_kusama(chain, referendum_index, vote, balance, conviction)
                    .await
            }
        }
    }

    async fn vote_polkadot(
        &self,
        chain: &Chain,
        referendum_index: u32,
        vote: Option<bool>,
        balance: u128,
        conviction: u8,
    ) -> anyhow::Result<(String, u64, u32)> {
        use polkadot::conviction_voting::calls::types::vote::Vote as VoteCall;
        use polkadot::proxy::calls::types::proxy::Call;
        use polkadot::runtime_types::pallet_conviction_voting::pallet::Call as ConvictionVotingCall;
        use polkadot::runtime_types::pallet_conviction_voting::vote::Vote;

        let real = AccountId32::from_str(&self.config.voter.polkadot_real_account_address)?;
        let uri = SecretUri::from_str(&self.config.voter.polkadot_proxy_account_seed_phrase)
            .expect("Invalid seed phrase.");
        let keypair = sr25519::Keypair::from_uri(&uri).expect("Invalid keypair.");

        let vote = if let Some(aye) = vote {
            VoteCall::Standard {
                vote: Vote(conviction + if aye { 128 } else { 0 }),
                balance,
            }
        } else {
            VoteCall::SplitAbstain {
                aye: 0,
                nay: 0,
                abstain: balance,
            }
        };
        let call = ConvictionVotingCall::vote {
            poll_index: referendum_index,
            vote,
        };
        let call = Call::ConvictionVoting(call);
        let proxy = polkadot::tx().proxy().proxy(real.into(), None, call);

        let api = OnlineClient::<PolkadotConfig>::from_url(&chain.rpc_url).await?;
        let tx_progress = api
            .tx()
            .sign_and_submit_then_watch_default(&proxy, &keypair)
            .await?;
        let tx_in_block = tx_progress.wait_for_finalized().await?;
        let block_hash = tx_in_block.block_hash();
        let block_hash = format!("0x{}", hex::encode(block_hash.0));
        let events = tx_in_block.wait_for_success().await?;
        let subtrate_client = SubstrateClient::new(
            &chain.rpc_url,
            self.config.substrate.connection_timeout_seconds,
            self.config.substrate.request_timeout_seconds,
        )
        .await?;
        let header = subtrate_client.get_block_header(&block_hash).await?;
        let block_number = header.get_number()?;
        let extrinsic_index = events.extrinsic_index();
        Ok((block_hash, block_number, extrinsic_index))
    }

    async fn vote_kusama(
        &self,
        chain: &Chain,
        referendum_index: u32,
        vote: Option<bool>,
        balance: u128,
        conviction: u8,
    ) -> anyhow::Result<(String, u64, u32)> {
        use kusama::conviction_voting::calls::types::vote::Vote as VoteCall;
        use kusama::proxy::calls::types::proxy::Call;
        use kusama::runtime_types::pallet_conviction_voting::pallet::Call as ConvictionVotingCall;
        use kusama::runtime_types::pallet_conviction_voting::vote::Vote;

        let real = AccountId32::from_str(&self.config.voter.kusama_real_account_address)?;
        let uri = SecretUri::from_str(&self.config.voter.kusama_proxy_account_seed_phrase)
            .expect("Invalid seed phrase.");
        let keypair = sr25519::Keypair::from_uri(&uri).expect("Invalid keypair.");

        let vote = if let Some(aye) = vote {
            VoteCall::Standard {
                vote: Vote(conviction + if aye { 128 } else { 0 }),
                balance,
            }
        } else {
            VoteCall::SplitAbstain {
                aye: 0,
                nay: 0,
                abstain: balance,
            }
        };
        let call = ConvictionVotingCall::vote {
            poll_index: referendum_index,
            vote,
        };
        let call = Call::ConvictionVoting(call);
        let proxy = kusama::tx().proxy().proxy(real.into(), None, call);

        let api = OnlineClient::<PolkadotConfig>::from_url(&chain.rpc_url).await?;
        let tx_progress = api
            .tx()
            .sign_and_submit_then_watch_default(&proxy, &keypair)
            .await?;
        let tx_in_block = tx_progress.wait_for_finalized().await?;
        let block_hash = tx_in_block.block_hash();
        let block_hash = format!("0x{}", hex::encode(block_hash.0));
        let events = tx_in_block.wait_for_success().await?;
        let subtrate_client = SubstrateClient::new(
            &chain.rpc_url,
            self.config.substrate.connection_timeout_seconds,
            self.config.substrate.request_timeout_seconds,
        )
        .await?;
        let header = subtrate_client.get_block_header(&block_hash).await?;
        let block_number = header.get_number()?;
        let extrinsic_index = events.extrinsic_index();
        Ok((block_hash, block_number, extrinsic_index))
    }
}
