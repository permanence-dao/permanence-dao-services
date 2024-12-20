use once_cell::sync::OnceCell;
use pdao_service::Service;
use pdao_telegram_bot::TelegramBot;

static SERVICE: OnceCell<TelegramBot> = OnceCell::new();

#[tokio::main]
async fn main() {
    let _ = SERVICE.set(TelegramBot::new().await.unwrap());
    SERVICE.get().unwrap().start().await;
}
