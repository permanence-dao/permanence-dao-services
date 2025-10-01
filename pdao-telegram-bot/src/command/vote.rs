use crate::command::util::{
    get_vote_counts, require_db_referendum, require_db_referendum_is_active,
    require_opensquare_cid, require_opensquare_referendum, require_opensquare_votes,
    require_subsquare_referendum, require_subsquare_referendum_active, require_thread,
    require_voting_admin,
};
use crate::TelegramBot;
use pdao_types::governance::policy::{VotingPolicy, VotingPolicyEvaluation};
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
        let voting_policy = VotingPolicy::voting_policy_for_track(&db_referendum.track);
        let (aye_count, nay_count, abstain_count) = get_vote_counts(&opensquare_votes);
        let past_votes = self.postgres.get_referendum_votes(db_referendum.id).await?;
        let vote = voting_policy.evaluate(voting_member_count, aye_count, nay_count, abstain_count);
        let mut message = format!("{voting_member_count} available members.");
        message = format!("{message}\n🟢 {aye_count} • 🔴 {nay_count} • ⚪️ {abstain_count}");
        let evaluation = match vote {
            VotingPolicyEvaluation::AbstainThresholdNotMet {
                abstain_threshold, ..
            } => format!(
                "{} is abstain before a total of {} votes.\n**Vote #{}: ⚪ ABSTAIN",
                db_referendum.track.name(),
                abstain_threshold,
                past_votes.len() + 1,
            ),
            VotingPolicyEvaluation::ParticipationNotMet {
                participation_threshold,
                ..
            } => {
                message = format!(
                    "{message}\n{} is no vote before a total of {} votes.\n➖ NO VOTE",
                    db_referendum.track.name(),
                    participation_threshold,
                );
                self.telegram_client
                    .send_message(
                        chat_id,
                        Some(thread_id),
                        &message,
                    )
                    .await?;
                return Ok(());
            },
            VotingPolicyEvaluation::MajorityAbstain {
                abstain_count,
                majority_threshold,
                ..
            } => format!(
                "{} abstains, more than the abstain threshold of {} votes.\n**Vote #{}: ⚪ ABSTAIN",
                abstain_count,
                majority_threshold,
                past_votes.len() + 1,
            ),
            VotingPolicyEvaluation::AyeAbstainMajorityAbstain {
                aye_count,
                abstain_count,
                majority_threshold,
                ..
            } => format!(
                "{} ayes & abstains, more than the {:.2}% majority threshold of {} votes.\n**Vote #{}: ⚪ ABSTAIN",
                aye_count + abstain_count,
                voting_policy.majority_percent,
                majority_threshold,
                past_votes.len() + 1,
            ),
            VotingPolicyEvaluation::AyeEqualsNayAbstain { .. } => format!(
                "Ayes are equal to nays with no abstains.\n**Vote #{}: ⚪ ABSTAIN",
                past_votes.len() + 1,
            ),
            VotingPolicyEvaluation::Aye {
                majority_threshold, ..
            } => format!(
                "{} ayes greater than the {:.2}% majority threshold ({} votes) for {}.\n**Vote #{}: 🟢 AYE",
                aye_count,
                voting_policy.majority_percent,
                majority_threshold,
                db_referendum.track.name(),
                past_votes.len() + 1,
            ),
            VotingPolicyEvaluation::Nay {
                ..
            } => format!(
                "{} aye or abstain requirements not met.\n**Vote #{}: 🔴 NAY",
                db_referendum.track.name(),
                past_votes.len() + 1,
            ),
        };
        message = format!("{message}\n{evaluation}");

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
                vote.simplify()?,
                balance,
                conviction,
            )
            .await?;
        let (subsquare_cid, subsquare_index) = if post_feedback {
            log::info!("Get OpenAI feedback summary.");
            let feedback = self
                .openai_client
                .fetch_feedback_summary(&chain, &subsquare_referendum, vote, &opensquare_votes)
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
                        &db_referendum.track,
                        &voting_policy,
                        past_votes.len() as u32,
                        voting_member_count,
                        &vote,
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
                        &db_referendum.track,
                        &voting_policy,
                        past_votes.len() as u32,
                        voting_member_count,
                        &vote,
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
                vote.simplify()?,
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
        message = format!(
            "{message}\n{voting_member_count} available members.\n{coi_message}\nhttps://{}.subscan.io/extrinsic/{}-{}",
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
