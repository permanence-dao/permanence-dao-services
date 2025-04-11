use crate::command::util::{require_thread, require_voting_admin};
use crate::{TelegramBot, CONFIG};
use pdao_types::substrate::chain::Chain;

impl TelegramBot {
    pub(crate) async fn process_archive_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        username: &str,
    ) -> anyhow::Result<()> {
        require_voting_admin(username)?;
        let thread_id = require_thread(thread_id)?;
        let (maybe_referendum_id, maybe_title) = if let Some(db_referendum) = self
            .postgres
            .get_referendum_by_telegram_chat_and_thread_id(chat_id, thread_id)
            .await?
        {
            (
                Some(db_referendum.id),
                Some(format!(
                    "[{}] {} #{} - {}",
                    db_referendum.track.short_name(),
                    Chain::from_id(db_referendum.network_id).token_ticker,
                    db_referendum.index,
                    db_referendum.title.unwrap_or("N/A".to_string()),
                )),
            )
        } else {
            (None, None)
        };

        use std::fs;
        use std::process::Command;
        let output = Command::new(&CONFIG.archive.python_bin_path) // Use the Python inside `venv`
            .arg(&CONFIG.archive.script_path)
            .arg(&CONFIG.telegram.api_id)
            .arg(&CONFIG.telegram.api_hash)
            .arg(CONFIG.telegram.chat_id.to_string())
            .arg(thread_id.to_string())
            .arg(&CONFIG.archive.temp_file_dir_path)
            .current_dir(&CONFIG.archive.working_dir_path)
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let file_path = if output.status.success() {
            stdout
        } else {
            self.telegram_client
                .send_message(chat_id, Some(thread_id), "Error while archiving topic.")
                .await?;
            return Ok(());
        };
        let file_path = file_path.trim();
        let archive_thread_id =
            if let Some(archive_thread_id) = self.postgres.get_archive_thread_id().await? {
                archive_thread_id
            } else {
                let archive_thread_id = self.telegram_client.create_archive_topic(&CONFIG).await?;
                self.postgres
                    .set_archive_thread_id(archive_thread_id)
                    .await?;
                archive_thread_id
            };
        log::info!("Archived file: {}", file_path);
        self.telegram_client
            .upload_file(
                file_path,
                CONFIG.telegram.chat_id,
                archive_thread_id,
                maybe_title.as_deref(),
            )
            .await?;
        log::info!("Uploaded archive to Telegram.");
        if let Some(referendum_id) = maybe_referendum_id {
            let message_archive = fs::read_to_string(file_path)?;
            self.postgres
                .save_referendum_message_archive(referendum_id, &message_archive)
                .await?;
            log::info!("Saved message archive into the database.");
        }
        self.telegram_client
            .delete_referendum_topic(CONFIG.telegram.chat_id, thread_id)
            .await?;
        log::info!("Deleted Telegram topic.");
        Ok(())
    }
}
