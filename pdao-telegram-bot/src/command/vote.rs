use crate::command::util::{
    get_vote_counts, require_db_referendum, require_db_referendum_is_active,
    require_opensquare_cid, require_opensquare_referendum, require_opensquare_votes,
    require_subsquare_referendum, require_subsquare_referendum_active, require_thread,
    require_voting_admin,
};
use crate::TelegramBot;
use pdao_types::governance::policy::Policy;
use pdao_types::substrate::chain::Chain;

impl TelegramBot {
    #[allow(clippy::cognitive_complexity)]
    pub(crate) async fn process_vote_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        username: &str,
        post_feedback: bool,
    ) -> anyhow::Result<()> {
        require_voting_admin(username)?;
        let thread_id = require_thread(thread_id)?;
        let db_referendum = require_db_referendum(&self.postgres, chat_id, thread_id).await?;
        require_db_referendum_is_active(&db_referendum)?;
        let opensquare_cid = require_opensquare_cid(&db_referendum)?;
        let chain = Chain::from_id(db_referendum.network_id);
        let voting_member_count = self.postgres.get_all_members(false).await?.len() as u32;
        let subsquare_referendum =
            require_subsquare_referendum(&self.subsquare_client, &chain, db_referendum.index)
                .await?;
        require_subsquare_referendum_active(&subsquare_referendum)?;
        let opensquare_referendum =
            require_opensquare_referendum(&self.opensquare_client, opensquare_cid).await?;
        let member_account_ids = self
            .postgres
            .get_all_member_account_ids_for_chain(true, Chain::polkadot().id)
            .await?;
        let opensquare_votes =
            require_opensquare_votes(&self.opensquare_client, opensquare_cid, &member_account_ids)
                .await?;
        let policy = Policy::policy_for_track(&db_referendum.track);
        let vote_counts = get_vote_counts(voting_member_count, &opensquare_votes);
        let past_votes = self.postgres.get_referendum_votes(db_referendum.id).await?;
        let (evaluation, description_lines) = policy.evaluate(&vote_counts);

        self.telegram_client
            .send_message(
                chat_id,
                Some(thread_id),
                "⚙️ Preparing the on-chain submission. Please give me some time.",
            )
            .await?;
        log::info!(
            "Vote #{} for {} referendum {}.",
            past_votes.len() + 1,
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
                evaluation.simplify()?,
                balance,
                conviction,
            )
            .await?;
        let (subsquare_cid, subsquare_index) = if post_feedback {
            log::info!("Get OpenAI feedback summary.");
            let feedback = self
                .openai_client
                .fetch_feedback_summary(
                    &chain,
                    &subsquare_referendum,
                    &evaluation,
                    &opensquare_votes,
                )
                .await?;
            log::info!("Post SubSquare comment.");
            let (cid, index) = if let Some(Some(first_vote_cid)) = past_votes
                .first()
                .map(|first_vote| first_vote.subsquare_comment_cid.clone())
            {
                let response = self
                    .subsquare_client
                    .post_comment_reply(
                        &chain,
                        &subsquare_referendum,
                        opensquare_cid,
                        &first_vote_cid,
                        past_votes.len() as u32,
                        &evaluation,
                        &description_lines,
                        db_referendum.has_coi,
                        &feedback,
                    )
                    .await?;
                (response.cid, response.index)
            } else {
                let response = self
                    .subsquare_client
                    .post_comment(
                        &chain,
                        &subsquare_referendum,
                        opensquare_cid,
                        past_votes.len() as u32,
                        &evaluation,
                        &description_lines,
                        db_referendum.has_coi,
                        &feedback,
                    )
                    .await?;
                (response.cid, response.index)
            };
            (Some(cid), Some(index))
        } else {
            log::info!("Skip SubSquare comment.");
            (None, None)
        };
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
                evaluation.simplify()?,
                balance,
                conviction,
                subsquare_cid.as_deref(),
                subsquare_index,
                db_referendum.has_coi,
                false,
            )
            .await?;
        for member_vote in opensquare_votes.iter() {
            self.postgres
                .save_member_vote(
                    vote_id as u32,
                    &member_vote.cid,
                    db_referendum.network_id,
                    db_referendum.id,
                    db_referendum.index,
                    &member_vote.address.to_ss58_check(),
                    &member_vote.remark,
                )
                .await?;
        }
        self.postgres
            .set_referendum_last_vote_id(db_referendum.id, Some(vote_id as u32))
            .await?;
        let coi_message = if db_referendum.has_coi {
            "CoI reported. DV delegation voted abstain."
        } else {
            "No CoI reported. DV delegation exercised."
        };
        let mut message = format!(
            "{}\n{coi_message}\nhttps://{}.subscan.io/extrinsic/{}-{}",
            description_lines.join("\n"),
            chain.chain.to_lowercase(),
            block_number,
            extrinsic_index,
        );
        if let Some(subsquare_index) = subsquare_index {
            message = format!(
                "{message}\nhttps://{}.subsquare.io/referenda/{}#{subsquare_index}",
                chain.chain.to_lowercase(),
                db_referendum.index,
            );
        } else {
            message = format!("{message}\nFeedback skipped.",);
        }

        self.telegram_client
            .update_referendum_topic_name(
                chat_id,
                thread_id,
                &opensquare_referendum.title,
                db_referendum.has_coi,
                None,
                &format!("V{}", past_votes.len() + 1),
                "🗳",
            )
            .await?;
        self.telegram_client
            .send_message(chat_id, Some(thread_id), &message)
            .await?;
        self.opensquare_client
            .make_appendant_on_proposal(&chain, opensquare_cid, &message)
            .await?;
        Ok(())
    }
}
