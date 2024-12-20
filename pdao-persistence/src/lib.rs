use lazy_static::lazy_static;
use pdao_config::Config;

pub mod postgres;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}
