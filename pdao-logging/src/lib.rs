#![warn(clippy::disallowed_types)]
use env_logger::{Builder, Env, Target, WriteStyle};
use pdao_config::Config;
use std::str::FromStr;

pub fn init(config: &Config) {
    let other_modules_log_level = log::LevelFilter::from_str(config.log.other_level.as_str())
        .expect("Cannot read log level configuration for outside modules.");
    let log_level = log::LevelFilter::from_str(config.log.pdao_level.as_str())
        .expect("Cannot read log level configuration for FTD modules.");
    let mut builder = Builder::from_env(Env::default());
    builder.target(Target::Stdout);
    builder.filter(None, other_modules_log_level);
    builder.filter(Some("pdao_metrics"), log_level);
    builder.filter(Some("pdao_metrics_server"), log_level);
    builder.filter(Some("pdao_opensquare_client"), log_level);
    builder.filter(Some("pdao_persistence"), log_level);
    builder.filter(Some("pdao_referendum_importer"), log_level);
    builder.filter(Some("pdao_subsquare_client"), log_level);
    builder.filter(Some("pdao_substrate_client"), log_level);
    builder.filter(Some("pdao_telegram_bot"), log_level);
    builder.filter(Some("pdao_telegram_client"), log_level);
    builder.filter(Some("pdao_types"), log_level);
    builder.filter(Some("pdao_voter"), log_level);
    builder.write_style(WriteStyle::Always);
    builder.init();
}
