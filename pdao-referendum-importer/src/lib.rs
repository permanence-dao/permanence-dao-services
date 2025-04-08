use pdao_config::Config;

use pdao_opensquare_client::OpenSquareClient;
use pdao_persistence::postgres::PostgreSQLStorage;
use pdao_subsquare_client::SubSquareClient;
use pdao_telegram_client::TelegramClient;
use pdao_types::governance::Referendum;
use pdao_types::substrate::chain::Chain;

#[derive(thiserror::Error, Clone, Debug)]
pub enum ReferendumImportError {
    #[error("Referendum has already been imported.")]
    AlreadyImported,
    #[error("Referendum not found on SubSquare.")]
    ReferendumNotFoundOnSubSquare,
    #[error("System error: {0}")]
    SystemError(String),
}

pub struct ReferendumImporter {
    config: Config,
    postgres: PostgreSQLStorage,
    telegram_client: TelegramClient,
    opensquare_client: OpenSquareClient,
    subsquare_client: SubSquareClient,
}

fn system_err(error: anyhow::Error, description: &str) -> ReferendumImportError {
    log::error!("{description}: {:?}", error);
    ReferendumImportError::SystemError(description.to_string())
}

impl ReferendumImporter {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            config: config.clone(),
            postgres: PostgreSQLStorage::new(config).await?,
            telegram_client: TelegramClient::new(config),
            opensquare_client: OpenSquareClient::new(config)?,
            subsquare_client: SubSquareClient::new(config)?,
        })
    }

    pub async fn import_referendum(
        &self,
        chain: &Chain,
        index: u32,
        snapshot_height: u64,
    ) -> Result<Referendum, ReferendumImportError> {
        let snapshot_height = snapshot_height - 50;
        log::info!("Process {} referendum #{}.", chain.token_ticker, index,);
        let maybe_db_referendum = self
            .postgres
            .get_referendum_by_index(chain.id, index)
            .await
            .map_err(|error| {
                ReferendumImportError::SystemError(format!("Database error: {:?}", error))
            })?;
        if maybe_db_referendum.is_some() {
            log::info!(
                "{} referendum #{} exists in the database. Skip.",
                chain.token_ticker,
                index,
            );
            return Err(ReferendumImportError::AlreadyImported);
        }
        let referendum = if let Some(referendum) = self
            .subsquare_client
            .fetch_referendum(chain, index)
            .await
            .map_err(|error| {
                system_err(error, "Error while fetching the referendum from SubSquare.")
            })? {
            referendum
        } else {
            return Err(ReferendumImportError::ReferendumNotFoundOnSubSquare);
        };
        let new_opensquare_proposal_response = self
            .opensquare_client
            .create_new_opensquare_proposal(chain, snapshot_height, &referendum)
            .await
            .map_err(|error| system_err(error, "OpenSquare error."))?;
        let new_telegram_topic_response = self
            .telegram_client
            .create_referendum_topic(
                chain,
                &self.config,
                &referendum,
                &new_opensquare_proposal_response,
            )
            .await
            .map_err(|error| system_err(error, "Telegram error."))?;
        let result = self
            .postgres
            .save_referendum(
                chain.id,
                &referendum,
                &new_opensquare_proposal_response.cid,
                &new_opensquare_proposal_response.post_uid,
                self.config.telegram.chat_id,
                new_telegram_topic_response,
            )
            .await
            .map_err(|error| system_err(error, "Database error while saving the referendum."))?;
        log::info!(
            "{} referendum #{} saved into the database with id {}.",
            chain.token_ticker,
            referendum.referendum_index,
            result
        );
        if let Some(referendum) = self
            .postgres
            .get_referendum_by_index(chain.id, index)
            .await
            .map_err(|error| {
                system_err(
                    error,
                    "Database error while fetching the new referendum. View the logs.",
                )
            })?
        {
            Ok(referendum)
        } else {
            Err(ReferendumImportError::SystemError(
                "Saved referendum not found in the database. View the logs.".to_string(),
            ))
        }
    }
}
