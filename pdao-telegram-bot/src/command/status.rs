use crate::command::util::{
    get_vote_counts, require_db_referendum, require_db_referendum_is_active,
    require_opensquare_cid, require_opensquare_referendum, require_opensquare_votes,
    require_subsquare_referendum, require_thread,
};
use crate::TelegramBot;
use pdao_types::governance::policy::Policy;
use pdao_types::governance::ReferendumStatus;
use pdao_types::substrate::chain::Chain;

impl TelegramBot {
    pub(crate) async fn process_status_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
    ) -> anyhow::Result<()> {
        let thread_id = require_thread(thread_id)?;
        let db_referendum = require_db_referendum(&self.postgres, chat_id, thread_id).await?;
        require_db_referendum_is_active(&db_referendum)?;
        let chain = Chain::from_id(db_referendum.network_id);
        let voting_member_count = self.postgres.get_all_members(false).await?.len() as u32;
        let subsquare_referendum =
            require_subsquare_referendum(&self.subsquare_client, &chain, db_referendum.index)
                .await?;
        let opensquare_cid = require_opensquare_cid(&db_referendum)?;
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
        let block_number = subsquare_referendum.state.block.number;
        let maybe_blocks_left = match subsquare_referendum.state.status {
            ReferendumStatus::Deciding => {
                if let Some(decision_info) = &subsquare_referendum.onchain_data.info.decision_info {
                    if let Some(decision_start_block) = decision_info.decision_start_block_number {
                        let decision_end_block = decision_start_block
                            + subsquare_referendum.track_info.decision_period as u64;
                        Some(decision_end_block.saturating_sub(block_number))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            ReferendumStatus::Confirming => {
                if let Some(decision_info) = &subsquare_referendum.onchain_data.info.decision_info {
                    if let Some(confirm_start_block) = decision_info.confirm_start_block_number {
                        let confirm_end_block = confirm_start_block
                            + subsquare_referendum.track_info.confirm_period as u64;
                        Some(confirm_end_block.saturating_sub(block_number))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        };

        let mut message = format!("{}", subsquare_referendum.state.status);
        if let Some(blocks_left) = maybe_blocks_left {
            let seconds_left = blocks_left * chain.block_time_seconds as u64;
            let mut counted_seconds = 0;
            let days_left = seconds_left / 60 / 60 / 24;
            counted_seconds += days_left * 24 * 60 * 60;
            let hours_left = (seconds_left - counted_seconds) / 60 / 60;
            counted_seconds += hours_left * 60 * 60;
            let minutes_left = (seconds_left - counted_seconds) / 60;
            let mut components: Vec<String> = Vec::new();
            if days_left > 0 {
                components.push(format!("{days_left}d"));
            }
            if hours_left > 0 {
                components.push(format!("{hours_left}hr"));
            }
            if days_left == 0 && minutes_left > 0 {
                components.push(format!("{minutes_left}min"));
            }
            let time_left = components.join(" ");
            message = format!("{message}: {time_left} left");
        }
        if subsquare_referendum.state.status.is_ongoing() {
            message = format!(
                "{message}\n{}",
                if db_referendum.preimage_exists {
                    "📝 Preimage exists"
                } else {
                    "⚠️ No preimage"
                }
            )
        }

        let (_, description_lines) = policy.evaluate(&vote_counts);
        message = format!("{message}\n{}", description_lines.join("\n"));

        if opensquare_referendum.status.to_lowercase() != "active" {
            message = format!("{message}\n\nMirror referendum is terminated.");
        }
        self.telegram_client
            .send_message(chat_id, Some(thread_id), &message)
            .await?;
        Ok(())
    }
}
