use crate::command::util::{
    require_db_referendum, require_db_referendum_is_active, require_opensquare_cid,
    require_opensquare_referendum, require_subsquare_referendum,
    require_subsquare_referendum_active, require_thread, require_voting_admin,
};
use crate::TelegramBot;
use pdao_types::substrate::chain::Chain;

impl TelegramBot {
    pub(crate) async fn process_coi_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        has_coi: bool,
        username: &str,
    ) -> anyhow::Result<()> {
        require_voting_admin(username)?;
        let thread_id = require_thread(thread_id)?;
        let db_referendum = require_db_referendum(&self.postgres, chat_id, thread_id).await?;
        require_db_referendum_is_active(&db_referendum)?;
        if has_coi && db_referendum.has_coi {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum has already been marked for conflict of interest.",
                )
                .await?;
            return Ok(());
        } else if !has_coi && !db_referendum.has_coi {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum has not been marked for conflict of interest.",
                )
                .await?;
            return Ok(());
        }
        let chain = Chain::from_id(db_referendum.network_id);
        let subsquare_referendum =
            require_subsquare_referendum(&self.subsquare_client, &chain, db_referendum.index)
                .await?;
        require_subsquare_referendum_active(&subsquare_referendum)?;
        let opensquare_cid = require_opensquare_cid(&db_referendum)?;
        let opensquare_referendum =
            require_opensquare_referendum(&self.opensquare_client, opensquare_cid).await?;
        let vote_count = self
            .postgres
            .get_referendum_vote_count(db_referendum.id)
            .await?;
        self.postgres
            .set_referendum_has_coi(db_referendum.id, has_coi)
            .await?;
        self.telegram_client
            .update_referendum_topic_name(
                chat_id,
                thread_id,
                &opensquare_referendum.title,
                has_coi,
                None,
                &format!("V{vote_count}"),
                "🗳",
            )
            .await?;
        let message = if has_coi {
            "Referendum has been marked for conlict of interest.\nDV delegation account will vote abstain on this referendum."
        } else {
            "Conlict of interest has been removed from the referendum.\nDV delegation account will vote normally on this referendum."
        };
        self.telegram_client
            .send_message(chat_id, Some(thread_id), message)
            .await?;
        Ok(())
    }
}
