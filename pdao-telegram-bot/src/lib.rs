use async_trait::async_trait;
use frankenstein::{Message, Update, UpdateContent};
use lazy_static::lazy_static;
use pdao_config::Config;
use pdao_service::Service;

use pdao_opensquare_client::OpenSquareClient;
use pdao_persistence::postgres::PostgreSQLStorage;
use pdao_referendum_importer::{ReferendumImportError, ReferendumImporter};
use pdao_subsquare_client::SubSquareClient;
use pdao_telegram_client::TelegramClient;
use pdao_types::governance::ReferendumStatus;
use pdao_types::substrate::chain::Chain;
use regex::Regex;

mod command;
mod metrics;

lazy_static! {
    static ref CONFIG: Config = Config::default();
    static ref CMD_REGEX: Regex =
        Regex::new(r"^/([a-zA-Z0-9_]+[@a-zA-Z0-9_]?)(\s+[a-zA-Z0-9_-]+)*").unwrap();
    static ref CMD_ARG_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
    static ref SPLITTER_REGEX: Regex = Regex::new(r"\s+").unwrap();
}

pub struct TelegramBot {
    postgres: PostgreSQLStorage,
    opensquare_client: OpenSquareClient,
    telegram_client: TelegramClient,
    referendum_importer: ReferendumImporter,
}

impl TelegramBot {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            postgres: PostgreSQLStorage::new(&CONFIG).await?,
            opensquare_client: OpenSquareClient::new(&CONFIG)?,
            telegram_client: TelegramClient::new(&CONFIG),
            referendum_importer: ReferendumImporter::new(&CONFIG).await?,
        })
    }

    async fn process_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        command: &str,
        args: &[String],
    ) -> anyhow::Result<()> {
        log::info!(
            "Process command {} for chat {} with arguments: {:?}",
            command,
            chat_id,
            args,
        );
        match command {
            "/import" => {
                self.process_import_command(chat_id, thread_id, args)
                    .await?;
            }
            "/archive" => {
                // unimplemented
            }
            "/vote" => {
                // unimplemented
            }
            "/update" => {
                // unimplemented
            }
            "/status" => {
                self.process_status_command(chat_id, thread_id).await?;
            }
            _ => {
                // err - send message
            }
        }
        Ok(())
    }

    async fn process_text_message(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        text: &str,
    ) -> anyhow::Result<()> {
        if CMD_REGEX.is_match(text) {
            log::info!("New command: {}", text);
            let (command, arguments): (String, Vec<String>) = {
                let parts: Vec<String> = SPLITTER_REGEX.split(text).map(String::from).collect();
                (
                    parts[0].clone(),
                    parts[1..]
                        .iter()
                        .filter(|arg| CMD_ARG_REGEX.is_match(arg))
                        .cloned()
                        .collect(),
                )
            };
            let command = command.replace(&CONFIG.telegram.bot_username, "");
            self.process_command(chat_id, thread_id, &command, &arguments)
                .await?;
        }
        Ok(())
    }

    async fn process_message(&self, message: &Message) -> anyhow::Result<()> {
        // text message
        if let Some(text) = &message.text {
            self.process_text_message(message.chat.id, message.message_thread_id, text)
                .await?;
        }
        Ok(())
    }

    async fn process_update(&self, update: &Update) {
        match &update.content {
            UpdateContent::Message(message) => {
                if message.chat.id != CONFIG.telegram.chat_id {
                    return;
                }
                if let Err(error) = self.process_message(message).await {
                    let message = format!(
                        "Error while processing message #{}: {:?}",
                        message.message_id, error,
                    );
                    log::error!("{message}");
                    let _ = self
                        .telegram_client
                        .send_message(CONFIG.telegram.chat_id, None, &message)
                        .await;
                }
            }
            UpdateContent::CallbackQuery(_callback_query) => (),
            UpdateContent::ChatMember(_chat_member_updated) => (),
            UpdateContent::MyChatMember(_chat_member_updated) => (),
            _ => (),
        }
    }
}

async fn import_referenda() -> anyhow::Result<()> {
    let postgres = PostgreSQLStorage::new(&CONFIG).await?;
    let subsquare_client = SubSquareClient::new(&CONFIG)?;
    let telegram_client = TelegramClient::new(&CONFIG);
    let referendum_importer = ReferendumImporter::new(&CONFIG).await?;
    let chain = Chain::polkadot();
    let referenda = subsquare_client.fetch_referenda(&chain, 1, 50).await?;
    let mut imported_referendum_count = 0;
    for referendum in referenda.items.iter() {
        if let ReferendumStatus::Deciding = referendum.state.status {
            let db_referendum = postgres
                .get_referendum_by_index(chain.id, referendum.referendum_index)
                .await?;
            if db_referendum.is_none() {
                if referendum.content.is_none() {
                    let message = format!(
                        "{} referendum {} has entered the decision period but missing content. Temporarily skipping auto-import.",
                        chain.display,
                        referendum.referendum_index,
                    );
                    log::warn!("{message}");
                    telegram_client
                        .send_message(CONFIG.telegram.chat_id, None, &message)
                        .await?;
                    continue;
                }
                log::info!("Try to import referendum {}.", referendum.referendum_index);
                if let Err(error) = referendum_importer
                    .import_referendum(&chain, referendum.referendum_index)
                    .await
                {
                    let message = match error {
                        ReferendumImportError::AlreadyImported => format!(
                            "Error while auto-importing {} referendum {}. It has already been imported.",
                            // "{} referendum {} already imported. You can use the `update` command under the related topic to update refererendum status and contents.",
                            chain.display,
                            referendum.referendum_index,
                        ),
                        ReferendumImportError::ReferendumNotFoundOnSubSquare => format!(
                            "Error while auto-importing {} referendum {}. Referendum not found on SubSquare.",
                            chain.display, referendum.referendum_index,
                        ),
                        ReferendumImportError::SystemError(description) => format!(
                            "System error while auto-importing {} referendum {}: {description}",
                            chain.display, referendum.referendum_index,
                        ),
                    };
                    telegram_client
                        .send_message(CONFIG.telegram.chat_id, None, &message)
                        .await?;
                    log::error!("{message}");
                } else {
                    telegram_client
                        .send_message(
                            CONFIG.telegram.chat_id,
                            None,
                            &format!(
                                "{} referendum {} auto-imported successfully.",
                                chain.display, referendum.referendum_index,
                            ),
                        )
                        .await?;
                    imported_referendum_count += 1;
                    log::info!(
                        "{} referendum {} auto-imported successfully.",
                        chain.display,
                        referendum.referendum_index
                    );
                }
            }
        }
    }
    log::info!("Imported {} referenda.", imported_referendum_count);
    Ok(())
}

#[async_trait(? Send)]
impl Service for TelegramBot {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (
            CONFIG.metrics.host.as_str(),
            CONFIG.metrics.referendum_importer_port,
        )
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        log::info!("Telegram bot started.");
        let mut offset: Option<i64> = None;

        tokio::spawn(async move {
            let delay_seconds = 60 * 10;
            loop {
                if let Err(err) = import_referenda().await {
                    log::error!("Import referenda failed: {}", err);
                }
                log::info!("Sleep for {} seconds.", delay_seconds);
                tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
            }
        });
        loop {
            let result = self.telegram_client.get_updates(offset).await;
            match result {
                Ok(updates) => {
                    for update in updates {
                        offset = Some((update.update_id + 1).into());
                        self.process_update(&update).await;
                    }
                }
                Err(error) => {
                    log::error!("Error while receiving Telegram updates: {:?}", error);
                }
            }
        }
    }
}
