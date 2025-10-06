use crate::TelegramBot;
use pdao_referendum_importer::ReferendumImportError;
use pdao_types::substrate::chain::Chain;
use std::str::FromStr;

impl TelegramBot {
    pub(crate) async fn process_import_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        args: &[String],
        polkadot_snapshot_height: u64,
    ) -> anyhow::Result<()> {
        if args.is_empty() {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    "Please append the chain name or ticker, and referendum id to the command.",
                    true,
                )
                .await?;
            return Ok(());
        }
        let mut chain = Chain::polkadot();
        let index_arg_index = args.len() - 1;
        if args.len() >= 2 {
            chain = if let Ok(chain) = Chain::from_str(&args[0]) {
                chain
            } else {
                self.telegram_client
                    .send_message(
                        chat_id,
                        thread_id,
                        &format!(
                            "Unknown chain: {}. Please use one of the known chains (Polkadot, Kusama).",
                            args[1],
                        ),
                        true,
                    )
                    .await?;
                return Ok(());
            };
        }
        let index = if let Ok(index) = args[index_arg_index].parse() {
            index
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    &format!(
                        "Invalid referendum id: {}. Please enter a valid number.",
                        args[index_arg_index],
                    ),
                    true,
                )
                .await?;
            return Ok(());
        };
        let preimage_exists = match self.voter.get_referendum_lookup(&chain, index).await? {
            Some(lookup) => self.voter.get_preimage(&chain, &lookup).await?.is_some(),
            None => false,
        };
        if let Err(error) = self
            .referendum_importer
            .import_referendum(&chain, index, polkadot_snapshot_height, preimage_exists)
            .await
        {
            let message = match error {
                ReferendumImportError::AlreadyImported => format!(
                    "{} referendum {} has already been imported.",
                    // "{} referendum {} already imported. You can use the `update` command under the related topic to update refererendum status and contents.",
                    chain.display,
                    index,
                ),
                ReferendumImportError::ReferendumNotFoundOnSubSquare => format!(
                    "{} referendum {} not found on SubSquare.",
                    chain.display, index,
                ),
                ReferendumImportError::SystemError(description) => description,
            };
            self.telegram_client
                .send_message(chat_id, thread_id, &message, true)
                .await?;
            return Ok(());
        }
        self.telegram_client
            .send_message(
                chat_id,
                thread_id,
                &format!(
                    "{} referendum {} imported successfully.",
                    chain.display, index
                ),
                true,
            )
            .await?;
        Ok(())
    }
}
