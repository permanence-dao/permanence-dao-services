use async_trait::async_trait;
use frankenstein::{Message, Update, UpdateContent};
use lazy_static::lazy_static;
use pdao_config::Config;
use pdao_service::Service;

use pdao_referendum_importer::ReferendumImporter;
use pdao_telegram_client::TelegramClient;
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
    telegram_client: TelegramClient,
    referendum_importer: ReferendumImporter,
}

impl TelegramBot {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
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
                // unimplemented
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
