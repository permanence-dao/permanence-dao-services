use once_cell::sync::Lazy;
use pdao_metrics::registry::IntGauge;

const _METRIC_PREFIX: &str = "pdao_telegram_bot";

pub fn _indexed_finalized_block_number() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        pdao_metrics::registry::register_int_gauge(
            _METRIC_PREFIX,
            "indexed_finalized_block_number",
            "Number of the last processed block",
        )
        .unwrap()
    });
    METER.clone()
}
