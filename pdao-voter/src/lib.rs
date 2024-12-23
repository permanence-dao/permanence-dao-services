use subxt::{OnlineClient, PolkadotConfig};

pub struct Voter;

impl Voter {
    pub async fn vote(&self) -> anyhow::Result<()> {
        let _api = OnlineClient::<PolkadotConfig>::new().await?;
        Ok(())
    }
}
