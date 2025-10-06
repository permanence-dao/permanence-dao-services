use crate::command::util::{
    require_db_referendum, require_opensquare_referendum, require_opensquare_referendum_active,
    require_thread, require_voting_admin,
};
use crate::TelegramBot;
use pdao_types::substrate::chain::Chain;

impl TelegramBot {
    pub(crate) async fn process_terminate_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        username: &str,
        topic_status: &str,
        topic_emoji: &str,
    ) -> anyhow::Result<()> {
        require_voting_admin(username)?;
        let thread_id = require_thread(thread_id)?;
        let db_referendum = require_db_referendum(&self.postgres, chat_id, thread_id).await?;
        let chain = Chain::from_id(db_referendum.network_id);
        let opensquare_referendum =
            require_opensquare_referendum(&self.opensquare_client, &db_referendum.opensquare_cid)
                .await?;
        require_opensquare_referendum_active(&opensquare_referendum)?;
        self.opensquare_client
            .terminate_proposal(&chain, &db_referendum.opensquare_cid)
            .await?;
        self.postgres.terminate_referendum(db_referendum.id).await?;
        self.telegram_client
            .send_message(
                chat_id,
                Some(thread_id),
                "OpenSquare referendum terminated.",
                true,
            )
            .await?;
        let current_vote_count = self
            .postgres
            .get_referendum_vote_count(db_referendum.id)
            .await?;
        self.telegram_client
            .update_referendum_topic_name(
                chat_id,
                thread_id,
                &opensquare_referendum.title,
                db_referendum.has_coi,
                Some(topic_status),
                &format!("V{current_vote_count}"),
                topic_emoji,
            )
            .await?;
        Ok(())
    }
}
