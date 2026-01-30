use crate::{polkadot, Voter};
use pdao_substrate_client::SubstrateClient;
use pdao_types::substrate::chain::Chain;
use polkadot::runtime_types::asset_hub_polkadot_runtime::RuntimeCall;
use polkadot::runtime_types::pallet_conviction_voting::pallet::Call as VotingCall;
use polkadot::runtime_types::pallet_proxy::pallet::Call as ProxyCall;

use std::str::FromStr;
use subxt::utils::AccountId32;
use subxt::{OnlineClient, PolkadotConfig};
use subxt_signer::{sr25519, SecretUri};

impl Voter {
    fn get_polkadot_remove_vote_call(
        real_account_address: &str,
        referendum_index: u32,
    ) -> anyhow::Result<RuntimeCall> {
        let real = AccountId32::from_str(real_account_address)?;
        Ok(RuntimeCall::Proxy(ProxyCall::proxy {
            real: real.into(),
            force_proxy_type: None,
            call: Box::new(
                polkadot::proxy::calls::types::proxy::Call::ConvictionVoting(
                    VotingCall::remove_vote {
                        class: None,
                        index: referendum_index,
                    },
                ),
            ),
        }))
    }

    pub(crate) async fn remove_vote_polkadot(
        &self,
        chain: &Chain,
        referendum_index: u32,
    ) -> anyhow::Result<(String, u64, u32)> {
        let main_proxy_call = Self::get_polkadot_remove_vote_call(
            &self.config.voter.polkadot_real_account_address,
            referendum_index,
        )?;
        let dv_proxy_call = Self::get_polkadot_remove_vote_call(
            &self.config.voter.polkadot_dv_delegation_account_address,
            referendum_index,
        )?;

        let call = polkadot::tx()
            .utility()
            .batch_all(vec![main_proxy_call, dv_proxy_call]);
        let api = OnlineClient::<PolkadotConfig>::from_url(&chain.asset_hub_rpc_url).await?;
        let uri = SecretUri::from_str(&self.config.voter.polkadot_proxy_account_seed_phrase)
            .expect("Invalid seed phrase.");
        let keypair = sr25519::Keypair::from_uri(&uri).expect("Invalid keypair.");
        let tx_progress = api
            .tx()
            .sign_and_submit_then_watch_default(&call, &keypair)
            .await?;
        let tx_in_block = tx_progress.wait_for_finalized().await?;
        let block_hash = tx_in_block.block_hash();
        let block_hash = format!("0x{}", hex::encode(block_hash.0));
        let events = tx_in_block.wait_for_success().await?;
        let subtrate_client = SubstrateClient::new(
            &chain.asset_hub_rpc_url,
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
