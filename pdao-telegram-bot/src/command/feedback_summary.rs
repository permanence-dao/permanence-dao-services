use crate::command::util::{
    get_vote_counts, require_db_referendum, require_db_referendum_is_active,
    require_opensquare_cid, require_opensquare_votes, require_subsquare_referendum, require_thread,
};
use crate::TelegramBot;
use pdao_types::governance::policy::{Policy, PolicyEvaluation};
use pdao_types::substrate::chain::Chain;

impl TelegramBot {
    #[allow(clippy::cognitive_complexity)]
    pub(crate) async fn process_feedback_summary_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
    ) -> anyhow::Result<()> {
        let thread_id = require_thread(thread_id)?;
        let db_referendum = require_db_referendum(&self.postgres, chat_id, thread_id).await?;
        require_db_referendum_is_active(&db_referendum)?;
        let opensquare_cid = require_opensquare_cid(&db_referendum)?;
        let chain = Chain::from_id(db_referendum.network_id);
        let voting_member_count = self.postgres.get_all_members(false).await?.len() as u32;
        let subsquare_referendum =
            require_subsquare_referendum(&self.subsquare_client, &chain, db_referendum.index)
                .await?;
        let member_account_ids = self
            .postgres
            .get_all_member_account_ids_for_chain(true, Chain::polkadot().id)
            .await?;
        let opensquare_votes =
            require_opensquare_votes(&self.opensquare_client, opensquare_cid, &member_account_ids)
                .await?;
        if opensquare_votes.is_empty() {
            self.telegram_client
                .send_message(chat_id, Some(thread_id), "No votes yet.")
                .await?;
            return Ok(());
        }
        let comments_length = opensquare_votes
            .iter()
            .map(|vote| vote.remark.len())
            .sum::<usize>();
        if comments_length == 0 {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    &format!(
                        "{} votes cast, but no feedback left yet.",
                        opensquare_votes.len(),
                    ),
                )
                .await?;
            return Ok(());
        }
        let vote_counts = get_vote_counts(voting_member_count, &opensquare_votes);
        let voting_policy = Policy::policy_for_track(&db_referendum.track);
        let (evaluation, _) = voting_policy.evaluate(&vote_counts);
        if let PolicyEvaluation::ParticipationNotMet {
            participation_threshold,
            ..
        } = evaluation
        {
            self.telegram_client
                    .send_message(
                        chat_id,
                        Some(thread_id),
                        &format!(
                            "Required participation of at least {} votes for {} not met yet. No vote, no feedback.",
                            participation_threshold,
                            db_referendum.track.name(),
                        ),
                    )
                    .await?;
            return Ok(());
        }

        let summary = self
            .openai_client
            .fetch_feedback_summary(
                &chain,
                &subsquare_referendum,
                &evaluation,
                &opensquare_votes,
            )
            .await?;
        self.telegram_client
            .send_message(chat_id, Some(thread_id), &summary)
            .await?;
        Ok(())
    }
}
