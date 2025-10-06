use crate::command::util::{
    require_db_referendum, require_db_referendum_is_active, require_opensquare_referendum,
    require_subsquare_referendum, require_subsquare_referendum_active, require_thread,
    require_voting_admin,
};
use crate::TelegramBot;
use pdao_types::substrate::chain::Chain;

impl TelegramBot {
    pub(crate) async fn process_remove_vote_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        username: &str,
    ) -> anyhow::Result<()> {
        require_voting_admin(username)?;
        let thread_id = require_thread(thread_id)?;
        let db_referendum = require_db_referendum(&self.postgres, chat_id, thread_id).await?;
        require_db_referendum_is_active(&db_referendum)?;
        let last_vote_id = if let Some(last_vote_id) = db_referendum.last_vote_id {
            last_vote_id
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "No vote posted for this referendum yet, or the vote was removed.",
                    true,
                )
                .await?;
            return Ok(());
        };
        let opensquare_referendum =
            require_opensquare_referendum(&self.opensquare_client, &db_referendum.opensquare_cid)
                .await?;
        let chain = Chain::from_id(db_referendum.network_id);
        let subsquare_referendum =
            require_subsquare_referendum(&self.subsquare_client, &chain, db_referendum.index)
                .await?;
        require_subsquare_referendum_active(&subsquare_referendum)?;
        self.telegram_client
            .send_message(
                chat_id,
                Some(thread_id),
                "⚙️ Removing the on-chain vote. Please give me some time.",
                true,
            )
            .await?;
        let (_block_hash, block_number, extrinsic_index) =
            self.voter.remove_vote(&chain, db_referendum.index).await?;
        self.postgres
            .set_referendum_last_vote_id(db_referendum.id, None)
            .await?;
        self.postgres.set_vote_removed(last_vote_id).await?;
        let message = format!(
            "Removed on-chain vote.\nhttps://{}.subscan.io/extrinsic/{}-{}",
            chain.chain.to_lowercase(),
            block_number,
            extrinsic_index
        );
        self.opensquare_client
            .make_appendant_on_proposal(&chain, &db_referendum.opensquare_cid, &message)
            .await?;
        self.telegram_client
            .send_message(chat_id, Some(thread_id), &message, true)
            .await?;
        self.telegram_client
            .update_referendum_topic_name(
                chat_id,
                thread_id,
                &opensquare_referendum.title,
                db_referendum.has_coi,
                None,
                "VR",
                "🗳",
            )
            .await?;
        Ok(())
    }
}
