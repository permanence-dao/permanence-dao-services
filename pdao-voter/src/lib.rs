use async_trait::async_trait;
use lazy_static::lazy_static;
use pdao_config::Config;
use pdao_service::Service;

mod metrics;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct Voter;

#[async_trait(? Send)]
impl Service for Voter {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (CONFIG.metrics.host.as_str(), CONFIG.metrics.voter_port)
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        log::info!("Voter started.");
        let delay_seconds = CONFIG.common.recovery_retry_seconds;
        loop {
            log::info!("Run completed. Restart in {} seconds.", delay_seconds);
            tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
        }
    }
}
