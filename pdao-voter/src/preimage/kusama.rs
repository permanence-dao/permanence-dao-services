use crate::kusama;
use pdao_types::substrate::chain::Chain;
use pdao_types::substrate::referendum::ReferendumLookup;
use subxt::utils::H256;
use subxt::{OnlineClient, PolkadotConfig};

pub(crate) async fn get_preimage_kusama(
    chain: &Chain,
    lookup: &ReferendumLookup,
) -> anyhow::Result<Option<Vec<u8>>> {
    let api = OnlineClient::<PolkadotConfig>::from_url(&chain.rpc_url).await?;
    let query = kusama::storage()
        .preimage()
        .preimage_for((H256(lookup.hash), lookup.length));
    let result = api.storage().at_latest().await?.fetch(&query).await?;
    Ok(result.map(|p| p.0))
}
