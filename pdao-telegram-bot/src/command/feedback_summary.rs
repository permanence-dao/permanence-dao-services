use crate::command::util::{
    get_vote_counts, require_db_referendum, require_db_referendum_is_active,
    require_opensquare_cid, require_opensquare_votes, require_subsquare_referendum, require_thread,
    require_voting_policy,
};
use crate::TelegramBot;
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
        if opensquare_votes.len() < 2 {
            self.telegram_client
                .send_message(chat_id, Some(thread_id), "There are less than 2 votes.")
                .await?;
            return Ok(());
        }

        let voting_policy = require_voting_policy(&db_referendum.track)?;
        let (aye_count, nay_count, abstain_count) = get_vote_counts(&opensquare_votes);
        let participation = aye_count + nay_count + abstain_count;
        let abstain_threshold =
            voting_policy.abstain_threshold_percent as u32 * voting_member_count / 100;
        let participation_threshold =
            voting_policy.participation_percent as u32 * voting_member_count / 100;
        let quorum_threshold = voting_policy.quorum_percent as u32 * voting_member_count / 100;
        let majority_threshold =
            voting_policy.majority_percent as u32 * (aye_count + nay_count) / 100;

        let vote = if abstain_count > abstain_threshold || participation < participation_threshold {
            None
        } else if aye_count < quorum_threshold || aye_count <= majority_threshold {
            Some(false)
        } else {
            Some(true)
        };

        let summary = self
            .openai_client
            .fetch_feedback_summary(&chain, &subsquare_referendum, vote, &opensquare_votes)
            .await?;
        self.telegram_client
            .send_message(chat_id, Some(thread_id), &summary)
            .await?;
        Ok(())
    }
}
