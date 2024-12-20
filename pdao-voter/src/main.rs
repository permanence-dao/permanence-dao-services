use lazy_static::lazy_static;
use pdao_service::Service;
use pdao_voter::Voter;

lazy_static! {
    static ref SERVICE: Voter = Voter;
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}
