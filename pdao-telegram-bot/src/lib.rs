use async_trait::async_trait;
use frankenstein::{Message, Update, UpdateContent};
use lazy_static::lazy_static;
use pdao_config::Config;
use pdao_service::Service;

use pdao_openai_client::OpenAIClient;
use pdao_opensquare_client::OpenSquareClient;
use pdao_persistence::postgres::PostgreSQLStorage;
use pdao_referendum_importer::{ReferendumImportError, ReferendumImporter};
use pdao_subsquare_client::SubSquareClient;
use pdao_substrate_client::SubstrateClient;
use pdao_telegram_client::TelegramClient;
use pdao_types::governance::subsquare::SubSquareReferendum;
use pdao_types::governance::{Referendum, ReferendumStatus};
use pdao_types::substrate::chain::Chain;
use pdao_voter::Voter;
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
    subsquare_client: SubSquareClient,
    telegram_client: TelegramClient,
    openai_client: OpenAIClient,
    referendum_importer: ReferendumImporter,
    voter: Voter,
}

impl TelegramBot {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            postgres: PostgreSQLStorage::new(&CONFIG).await?,
            opensquare_client: OpenSquareClient::new(&CONFIG)?,
            subsquare_client: SubSquareClient::new(&CONFIG)?,
            telegram_client: TelegramClient::new(&CONFIG),
            openai_client: OpenAIClient::new(&CONFIG)?,
            referendum_importer: ReferendumImporter::new(&CONFIG).await?,
            voter: Voter::new(&CONFIG).await?,
        })
    }

    async fn process_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        username: &str,
        command: &str,
        args: &[String],
    ) -> anyhow::Result<()> {
        log::info!(
            "Process command {} for chat {} thread {:?} with arguments: {:?}",
            command,
            chat_id,
            thread_id,
            args,
        );
        match command {
            "/archive" => {
                self.process_archive_command(chat_id, thread_id, username)
                    .await?;
            }
            "/forceabstain" => {
                self.process_force_vote_command(chat_id, thread_id, username, None)
                    .await?;
            }
            "/forceaye" => {
                self.process_force_vote_command(chat_id, thread_id, username, Some(true))
                    .await?;
            }
            "/forcenay" => {
                self.process_force_vote_command(chat_id, thread_id, username, Some(false))
                    .await?;
            }
            "/import" => {
                let polkadot_snapshot_height = get_polkadot_snapshot_height().await?;
                self.process_import_command(chat_id, thread_id, args, polkadot_snapshot_height)
                    .await?;
            }
            "/removevote" => {
                self.process_remove_vote_command(chat_id, thread_id, username)
                    .await?
            }
            "/status" => {
                self.process_status_command(chat_id, thread_id).await?;
            }
            "/terminate" => {
                self.process_terminate_command(chat_id, thread_id, username, "DONE", "✅")
                    .await?;
            }
            "/timeout" => {
                self.process_terminate_command(chat_id, thread_id, username, "MISSED", "🏁")
                    .await?;
            }
            "/vote" => {
                self.process_vote_command(chat_id, thread_id, username, true)
                    .await?;
            }
            "/votewithoutfeedback" => {
                self.process_vote_command(chat_id, thread_id, username, false)
                    .await?;
            }
            "/notify" => {
                self.process_notify_command(chat_id, thread_id, username)
                    .await?;
            }
            "/reportcoi" => {
                self.process_coi_command(chat_id, thread_id, true, username)
                    .await?;
            }
            "/removecoi" => {
                self.process_coi_command(chat_id, thread_id, false, username)
                    .await?;
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
        username: &str,
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
            self.process_command(chat_id, thread_id, username, &command, &arguments)
                .await?;
        } /* else if thread_id == Some(CONFIG.telegram.bot_chat_thread_id) {
              let response = self.openai_client.fetch_chat_response(username, text).await?;
              self.telegram_client.send_message(
                  chat_id,
                  thread_id,
                  &response,
              ).await?;
          } */
        Ok(())
    }

    async fn process_message(&self, message: &Message) -> anyhow::Result<()> {
        // text message
        if let Some(Some(username)) = &message.from.as_ref().map(|a| a.username.as_ref()) {
            if let Some(text) = &message.text {
                self.process_text_message(
                    message.chat.id,
                    message.message_thread_id,
                    username,
                    text,
                )
                .await?;
            }
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
                    let thread_id = message.message_thread_id;
                    let message = format!(
                        "Error while processing message #{}: {:?}",
                        message.message_id, error,
                    );
                    log::error!("{message}");
                    let _ = self
                        .telegram_client
                        .send_message(CONFIG.telegram.chat_id, thread_id, &message)
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

async fn get_polkadot_snapshot_height() -> anyhow::Result<u64> {
    let polkadot = Chain::polkadot();
    let substrate_client = SubstrateClient::new(
        &polkadot.rpc_url,
        CONFIG.substrate.connection_timeout_seconds,
        CONFIG.substrate.request_timeout_seconds,
    )
    .await?;
    substrate_client.get_finalized_block_number().await
}

async fn import_referendum(
    referendum_importer: &ReferendumImporter,
    telegram_client: &TelegramClient,
    chain: &Chain,
    referendum: &SubSquareReferendum,
) -> anyhow::Result<bool> {
    log::info!(
        "Try to import {} referendum {}.",
        chain.display,
        referendum.referendum_index
    );
    let snapshot_height = match chain.token_ticker.as_str() {
        "DOT" => referendum.state.block.number,
        _ => get_polkadot_snapshot_height().await?,
    };
    if let Err(error) = referendum_importer
        .import_referendum(chain, referendum.referendum_index, snapshot_height)
        .await
    {
        let message = match error {
            ReferendumImportError::AlreadyImported => format!(
                "Error while auto-importing {} referendum {}. It has already been imported.",
                chain.display, referendum.referendum_index,
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
        Ok(false)
    } else {
        telegram_client
            .send_message(
                CONFIG.telegram.chat_id,
                None,
                &format!(
                    "🗳️ {} referendum {} imported:\n{}",
                    chain.display,
                    referendum.referendum_index,
                    referendum
                        .title
                        .clone()
                        .unwrap_or("No title".to_string())
                        .replace("_", "\\_"),
                ),
            )
            .await?;
        log::info!(
            "{} referendum {} imported.",
            chain.display,
            referendum.referendum_index
        );
        Ok(true)
    }
}

async fn update_referendum_status(
    postgres: &PostgreSQLStorage,
    opensquare_client: &OpenSquareClient,
    telegram_client: &TelegramClient,
    db_referendum: &Referendum,
    subsquare_referendum: &SubSquareReferendum,
    chain: &Chain,
) -> anyhow::Result<()> {
    log::info!(
        "Update {} referendum #{} state: {} -> {}",
        chain.display,
        db_referendum.index,
        db_referendum.status,
        subsquare_referendum.state.status,
    );
    postgres
        .update_referendum_status(db_referendum.id, &subsquare_referendum.state.status)
        .await?;
    telegram_client
        .send_message(
            db_referendum.telegram_chat_id,
            Some(db_referendum.telegram_topic_id),
            &format!(
                "{} {}",
                subsquare_referendum.state.status.get_icon(),
                subsquare_referendum.state.status
            ),
        )
        .await?;
    if !db_referendum.is_terminated && subsquare_referendum.state.status.requires_termination() {
        let opensquare_cid = if let Some(opensquare_cid) = db_referendum.opensquare_cid.as_ref() {
            opensquare_cid
        } else {
            log::error!("Opensquare CID not found - exit.");
            return Ok(());
        };
        let opensquare_referendum = if let Some(opensquare_referendum) =
            opensquare_client.fetch_referendum(opensquare_cid).await?
        {
            opensquare_referendum
        } else {
            log::error!("Opensquare referendum not found - exit.");
            return Ok(());
        };
        log::info!("New status requires termination.");
        opensquare_client
            .terminate_opensquare_proposal(chain, opensquare_cid)
            .await?;
        postgres.terminate_referendum(db_referendum.id).await?;
        telegram_client
            .send_message(
                db_referendum.telegram_chat_id,
                Some(db_referendum.telegram_topic_id),
                "OpenSquare referendum terminated.",
            )
            .await?;
        let current_vote_count = postgres.get_referendum_vote_count(db_referendum.id).await?;
        telegram_client
            .update_referendum_topic_name(
                db_referendum.telegram_chat_id,
                db_referendum.telegram_topic_id,
                &opensquare_referendum.title,
                db_referendum.has_coi,
                Some(&subsquare_referendum.state.status.to_string().to_uppercase()),
                &format!("V{current_vote_count}"),
                "✅",
            )
            .await?;
    }
    Ok(())
}

async fn import_referenda(chain: &Chain) -> anyhow::Result<()> {
    let postgres = PostgreSQLStorage::new(&CONFIG).await?;
    let opensquare_client = OpenSquareClient::new(&CONFIG)?;
    let subsquare_client = SubSquareClient::new(&CONFIG)?;
    let telegram_client = TelegramClient::new(&CONFIG);
    let referendum_importer = ReferendumImporter::new(&CONFIG).await?;
    let referenda = subsquare_client.fetch_referenda(chain, 1, 50).await?;
    let mut imported_referendum_count = 0;
    for subsquare_referendum in referenda.items.iter() {
        let maybe_db_referendum = postgres
            .get_referendum_by_index(chain.id, subsquare_referendum.referendum_index)
            .await?;
        if let Some(db_referendum) = maybe_db_referendum.as_ref() {
            if db_referendum.status != subsquare_referendum.state.status
                && !db_referendum.is_archived
            {
                update_referendum_status(
                    &postgres,
                    &opensquare_client,
                    &telegram_client,
                    db_referendum,
                    subsquare_referendum,
                    chain,
                )
                .await?;
            }
        } else if (ReferendumStatus::Deciding == subsquare_referendum.state.status
            || ReferendumStatus::Confirming == subsquare_referendum.state.status)
            && import_referendum(
                &referendum_importer,
                &telegram_client,
                chain,
                subsquare_referendum,
            )
            .await?
        {
            imported_referendum_count += 1;
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

        let polkadot = Chain::polkadot();
        let kusama = Chain::kusama();
        tokio::spawn(async move {
            let delay_seconds = 60 * 30;
            loop {
                if let Err(err) = import_referenda(&polkadot).await {
                    log::error!("Import Polkadot referenda failed: {}", err);
                }
                if let Err(err) = import_referenda(&kusama).await {
                    log::error!("Import Kusama referenda failed: {}", err);
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
