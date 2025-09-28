use crate::kusama::{
    self,
    referenda::storage::types::referendum_info_for::ReferendumInfoFor as KusamaReferendumInfoFor,
    runtime_types::frame_support::traits::preimages::Bounded,
};
use pdao_types::substrate::{chain::Chain, referendum::ReferendumLookup};
use subxt::{OnlineClient, PolkadotConfig};

pub(crate) async fn get_referendum_info_kusama(
    chain: &Chain,
    referendum_index: u32,
) -> anyhow::Result<Option<KusamaReferendumInfoFor>> {
    let api = OnlineClient::<PolkadotConfig>::from_url(&chain.rpc_url).await?;
    let query = kusama::storage()
        .referenda()
        .referendum_info_for(referendum_index);
    let result = api.storage().at_latest().await?.fetch(&query).await?;
    Ok(result)
}

pub(crate) async fn get_referendum_lookup_kusama(
    chain: &Chain,
    referendum_index: u32,
) -> anyhow::Result<Option<ReferendumLookup>> {
    let referendum_info = get_referendum_info_kusama(chain, referendum_index).await?;
    let lookup = if let Some(referendum_info) = referendum_info {
        match referendum_info {
            KusamaReferendumInfoFor::Ongoing(a) => match a.proposal {
                Bounded::Lookup { hash, len } => Some(ReferendumLookup {
                    hash: hash.0,
                    length: len,
                }),
                _ => None,
            },
            _ => None,
        }
    } else {
        None
    };
    Ok(lookup)
}
