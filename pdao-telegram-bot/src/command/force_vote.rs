use crate::command::util::{
    require_db_referendum, require_db_referendum_is_active, require_opensquare_referendum,
    require_subsquare_referendum, require_subsquare_referendum_active, require_thread,
    require_voting_admin,
};
use crate::TelegramBot;
use pdao_types::substrate::chain::Chain;

impl TelegramBot {
    pub(crate) async fn process_force_vote_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        username: &str,
        vote: Option<bool>,
    ) -> anyhow::Result<()> {
        require_voting_admin(username)?;
        let thread_id = require_thread(thread_id)?;
        let db_referendum = require_db_referendum(&self.postgres, chat_id, thread_id).await?;
        require_db_referendum_is_active(&db_referendum)?;
        let chain = Chain::from_id(db_referendum.network_id);
        let subsquare_referendum =
            require_subsquare_referendum(&self.subsquare_client, &chain, db_referendum.index)
                .await?;
        require_subsquare_referendum_active(&subsquare_referendum)?;
        let opensquare_referendum =
            require_opensquare_referendum(&self.opensquare_client, &db_referendum.opensquare_cid)
                .await?;
        self.telegram_client
            .send_message(
                chat_id,
                Some(thread_id),
                "⚙️ Preparing the on-chain submission. Please give me some time.",
                true,
            )
            .await?;
        log::info!(
            "Force-{} for {} referendum {}.",
            if let Some(vote) = vote {
                if vote {
                    "aye"
                } else {
                    "nay"
                }
            } else {
                "abstain"
            },
            chain.chain,
            db_referendum.index
        );
        let balance = 10u128.pow(chain.token_decimals as u32);
        let conviction = 1;
        log::info!("Submit vote.");
        let (block_hash, block_number, extrinsic_index) = self
            .voter
            .vote(
                &chain,
                db_referendum.index,
                db_referendum.has_coi,
                vote,
                balance,
                conviction,
            )
            .await?;
        log::info!("Save vote in DB.");
        let vote_id = self
            .postgres
            .save_vote(
                db_referendum.network_id,
                db_referendum.id,
                db_referendum.index,
                &block_hash,
                block_number,
                extrinsic_index,
                vote,
                balance,
                conviction,
                None,
                None,
                db_referendum.has_coi,
                true,
            )
            .await?;
        self.postgres
            .set_referendum_last_vote_id(db_referendum.id, Some(vote_id as u32))
            .await?;
        let current_vote_count = self
            .postgres
            .get_referendum_vote_count(db_referendum.id)
            .await?;
        let message = format!(
            "**Vote #{}: FORCE-{}**\nhttps://{}.subscan.io/extrinsic/{}-{}",
            current_vote_count,
            (if let Some(vote) = vote {
                if vote {
                    "AYE"
                } else {
                    "NAY"
                }
            } else {
                "ABSTAIN"
            })
            .to_string()
            .to_uppercase(),
            chain.chain.to_lowercase(),
            block_number,
            extrinsic_index,
        );
        self.telegram_client
            .update_referendum_topic_name(
                chat_id,
                thread_id,
                &opensquare_referendum.title,
                db_referendum.has_coi,
                None,
                &format!("V{current_vote_count}"),
                "🗳",
            )
            .await?;
        self.opensquare_client
            .make_appendant_on_proposal(&chain, &db_referendum.opensquare_cid, &message)
            .await?;
        self.telegram_client
            .send_message(chat_id, Some(thread_id), &message, true)
            .await?;
        Ok(())
    }
}
