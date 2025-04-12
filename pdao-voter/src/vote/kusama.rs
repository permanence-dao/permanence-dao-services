use crate::{kusama, Voter};
use kusama::conviction_voting::calls::types::vote::Vote as VoteCall;
use kusama::runtime_types::pallet_conviction_voting::pallet::Call as VotingCall;
use kusama::runtime_types::pallet_conviction_voting::vote::Vote;
use kusama::runtime_types::pallet_proxy::pallet::Call as ProxyCall;
use kusama::runtime_types::staging_kusama_runtime::RuntimeCall as KusamaRuntimeCall;
use pdao_substrate_client::SubstrateClient;
use pdao_types::substrate::chain::Chain;
use std::str::FromStr;
use subxt::utils::AccountId32;
use subxt::{OnlineClient, PolkadotConfig};
use subxt_signer::{sr25519, SecretUri};

impl Voter {
    fn get_kusama_vote_call(
        real_account_address: &str,
        referendum_index: u32,
        vote: Option<bool>,
        balance: u128,
        conviction: u8,
    ) -> anyhow::Result<KusamaRuntimeCall> {
        let real = AccountId32::from_str(real_account_address)?;
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
        Ok(KusamaRuntimeCall::Proxy(ProxyCall::proxy {
            real: real.into(),
            force_proxy_type: None,
            call: Box::new(kusama::proxy::calls::types::proxy::Call::ConvictionVoting(
                VotingCall::vote {
                    poll_index: referendum_index,
                    vote,
                },
            )),
        }))
    }

    pub(crate) async fn vote_kusama(
        &self,
        chain: &Chain,
        referendum_index: u32,
        has_coi: bool,
        vote: Option<bool>,
        balance: u128,
        conviction: u8,
    ) -> anyhow::Result<(String, u64, u32)> {
        let main_proxy_call = Self::get_kusama_vote_call(
            &self.config.voter.kusama_real_account_address,
            referendum_index,
            vote,
            balance,
            conviction,
        )?;
        let dv_proxy_call = Self::get_kusama_vote_call(
            &self.config.voter.kusama_dv_delegation_account_address,
            referendum_index,
            if has_coi { None } else { vote },
            balance,
            conviction,
        )?;

        let call = kusama::tx()
            .utility()
            .batch_all(vec![main_proxy_call, dv_proxy_call]);
        let api = OnlineClient::<PolkadotConfig>::from_url(&chain.rpc_url).await?;
        let uri = SecretUri::from_str(&self.config.voter.kusama_proxy_account_seed_phrase)
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
