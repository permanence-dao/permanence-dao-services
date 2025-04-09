use crate::{TelegramBot, CONFIG};
use pdao_referendum_importer::ReferendumImportError;
use pdao_types::governance::opensquare::OpenSquareVote;
use pdao_types::governance::policy::VotingPolicy;
use pdao_types::governance::ReferendumStatus;
use pdao_types::substrate::account_id::AccountId;
use pdao_types::substrate::chain::Chain;
use std::str::FromStr;

impl TelegramBot {
    pub(crate) async fn process_import_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        args: &[String],
        polkadot_snapshot_height: u64,
    ) -> anyhow::Result<()> {
        if args.is_empty() {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    "Please append the chain name or ticker, and referendum id to the command.",
                )
                .await?;
            return Ok(());
        }
        let mut chain = Chain::polkadot();
        let index_arg_index = args.len() - 1;
        if args.len() >= 2 {
            chain = if let Ok(chain) = Chain::from_str(&args[0]) {
                chain
            } else {
                self.telegram_client
                    .send_message(
                        chat_id,
                        thread_id,
                        &format!(
                            "Unknown chain: {}. Please use one of the known chains (Polkadot, Kusama).",
                            args[1],
                        ),
                    )
                    .await?;
                return Ok(());
            };
        }
        let index = if let Ok(index) = args[index_arg_index].parse() {
            index
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    &format!(
                        "Invalid referendum id: {}. Please enter a valid number.",
                        args[index_arg_index],
                    ),
                )
                .await?;
            return Ok(());
        };
        if let Err(error) = self
            .referendum_importer
            .import_referendum(&chain, index, polkadot_snapshot_height)
            .await
        {
            let message = match error {
                ReferendumImportError::AlreadyImported => format!(
                    "{} referendum {} has already been imported.",
                    // "{} referendum {} already imported. You can use the `update` command under the related topic to update refererendum status and contents.",
                    chain.display,
                    index,
                ),
                ReferendumImportError::ReferendumNotFoundOnSubSquare => format!(
                    "{} referendum {} not found on SubSquare.",
                    chain.display, index,
                ),
                ReferendumImportError::SystemError(description) => description,
            };
            self.telegram_client
                .send_message(chat_id, thread_id, &message)
                .await?;
            return Ok(());
        }
        self.telegram_client
            .send_message(
                chat_id,
                thread_id,
                &format!(
                    "{} referendum {} imported successfully.",
                    chain.display, index
                ),
            )
            .await?;
        Ok(())
    }

    pub(crate) async fn process_status_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
    ) -> anyhow::Result<()> {
        let thread_id = if let Some(thread_id) = thread_id {
            thread_id
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    "This command can only be called from a referendum topic.",
                )
                .await?;
            return Ok(());
        };
        let db_referendum = if let Some(referendum) = self
            .postgres
            .get_referendum_by_telegram_chat_and_thread_id(chat_id, thread_id)
            .await?
        {
            if referendum.is_terminated {
                self.telegram_client
                    .send_message(chat_id, Some(thread_id), "Referendum has been terminated.")
                    .await?;
                return Ok(());
            }
            referendum
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum not found in the storage. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let chain = Chain::from_id(db_referendum.network_id);
        let subsquare_referendum = if let Some(referendum) = self
            .subsquare_client
            .fetch_referendum(&chain, db_referendum.index)
            .await?
        {
            referendum
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum not found on SubSquare. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let cid = if let Some(cid) = &db_referendum.opensquare_cid {
            cid
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "OpenSquare CID not found in the referendum record. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let opensquare_referendum = if let Some(opensquare_referendum) =
            self.opensquare_client.fetch_referendum(cid).await?
        {
            opensquare_referendum
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum not found on OpenSquare by CID. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let opensquare_votes = if let Some(opensquare_votes) =
            self.opensquare_client.fetch_referendum_votes(cid).await?
        {
            opensquare_votes
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum not found on OpenSquare by CID. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let voting_policy = if let Some(voting_policy) =
            VotingPolicy::voting_policy_for_track(db_referendum.track)
        {
            voting_policy
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    &format!(
                        "No voting policy is defined for {}.",
                        db_referendum.track.name(),
                    ),
                )
                .await?;
            return Ok(());
        };
        let mut aye_count = 0;
        let mut nay_count = 0;
        let mut abstain_count = 0;
        for vote in opensquare_votes {
            if vote.choices.contains(&OpenSquareVote::Aye) {
                aye_count += 1;
            } else if vote.choices.contains(&OpenSquareVote::Nay) {
                nay_count += 1;
            } else if vote.choices.contains(&OpenSquareVote::Abstain) {
                abstain_count += 1;
            }
        }

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
                components.push(format!("{}d", days_left));
            }
            if hours_left > 0 {
                components.push(format!("{}hr", hours_left));
            }
            if days_left == 0 && minutes_left > 0 {
                components.push(format!("{}min", minutes_left));
            }
            let time_left = components.join(" ");
            message = format!("{message}: {time_left} left");
        }
        message = format!("{message}\n🟢 {aye_count} • 🔴 {nay_count} • ⚪️ {abstain_count}");
        let participation = aye_count + nay_count + abstain_count;
        let participation_percent = (participation * 100) / CONFIG.voter.member_count;
        let quorum_percent = (aye_count * 100) / CONFIG.voter.member_count;
        let aye_percent = if (aye_count + nay_count) == 0 {
            0
        } else {
            (aye_count * 100) / (aye_count + nay_count)
        };
        message = if participation_percent < voting_policy.participation_percent as u32 {
            format!(
                "{message}\n{}% participation not met.\nABSTAIN",
                voting_policy.participation_percent,
            )
        } else if quorum_percent < voting_policy.quorum_percent as u32 {
            format!(
                "{message}\n{}% quorum not met.\nNAY",
                voting_policy.quorum_percent,
            )
        } else if aye_percent <= voting_policy.majority_percent as u32 {
            format!(
                "{message}\n{}% majority not met.\nNAY",
                voting_policy.majority_percent,
            )
        } else {
            format!("{message}\nAYE")
        };
        if opensquare_referendum.status.to_lowercase() != "active" {
            message = format!("{message}\nMirror referendum has been terminated.");
        }
        self.telegram_client
            .send_message(chat_id, Some(thread_id), &message)
            .await?;
        Ok(())
    }

    pub(crate) async fn process_terminate_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        username: &str,
        topic_status: &str,
        topic_emoji: &str,
    ) -> anyhow::Result<()> {
        if !CONFIG.voter.voting_admin_usernames.contains(username) {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    "This command can only be called by a voting admin.",
                )
                .await?;
            return Ok(());
        }
        let thread_id = if let Some(thread_id) = thread_id {
            thread_id
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    "This command can only be called from a referendum topic.",
                )
                .await?;
            return Ok(());
        };
        let db_referendum = if let Some(referendum) = self
            .postgres
            .get_referendum_by_telegram_chat_and_thread_id(chat_id, thread_id)
            .await?
        {
            referendum
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum not found in the storage. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let chain = Chain::from_id(db_referendum.network_id);
        let cid = if let Some(cid) = &db_referendum.opensquare_cid {
            cid
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "OpenSquare CID not found in the referendum record. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let opensquare_referendum = if let Some(opensquare_referendum) =
            self.opensquare_client.fetch_referendum(cid).await?
        {
            opensquare_referendum
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum not found on OpenSquare by CID. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        if opensquare_referendum.status != "active" {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "OpenSquare referendum is not active.",
                )
                .await?;
            return Ok(());
        }
        self.opensquare_client
            .terminate_opensquare_proposal(&chain, cid)
            .await?;
        self.postgres.terminate_referendum(db_referendum.id).await?;
        self.telegram_client
            .send_message(
                chat_id,
                Some(thread_id),
                "OpenSquare referendum terminated.",
            )
            .await?;
        let vote_count = self
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
                &format!("V{vote_count}"),
                topic_emoji,
            )
            .await?;
        Ok(())
    }

    pub(crate) async fn process_remove_vote_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        username: &str,
    ) -> anyhow::Result<()> {
        if !CONFIG.voter.voting_admin_usernames.contains(username) {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    "This command can only be called by a voting admin.",
                )
                .await?;
            return Ok(());
        }
        let thread_id = if let Some(thread_id) = thread_id {
            thread_id
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    "This command can only be called from a referendum topic.",
                )
                .await?;
            return Ok(());
        };
        let db_referendum = if let Some(referendum) = self
            .postgres
            .get_referendum_by_telegram_chat_and_thread_id(chat_id, thread_id)
            .await?
        {
            if referendum.is_terminated {
                self.telegram_client
                    .send_message(
                        chat_id,
                        Some(thread_id),
                        "Referendum has been terminated. Cannot remove vote.",
                    )
                    .await?;
                return Ok(());
            }
            referendum
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum not found in the storage. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let last_vote_id = if let Some(last_vote_id) = db_referendum.last_vote_id {
            last_vote_id
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "No vote posted for this referendum yet, or the vote was removed.",
                )
                .await?;
            return Ok(());
        };
        let cid = if let Some(cid) = &db_referendum.opensquare_cid {
            cid
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "OpenSquare CID not found in the referendum record. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let opensquare_referendum =
            if let Some(referendum) = self.opensquare_client.fetch_referendum(cid).await? {
                referendum
            } else {
                self.telegram_client
                    .send_message(
                        chat_id,
                        Some(thread_id),
                        "Referendum not found on OpenSquare. Contact admin.",
                    )
                    .await?;
                return Ok(());
            };
        let chain = Chain::from_id(db_referendum.network_id);
        let subsquare_referendum = if let Some(referendum) = self
            .subsquare_client
            .fetch_referendum(&chain, db_referendum.index)
            .await?
        {
            referendum
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum not found on SubSquare. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        if !(subsquare_referendum.state.status == ReferendumStatus::Deciding
            || subsquare_referendum.state.status == ReferendumStatus::Preparing
            || subsquare_referendum.state.status == ReferendumStatus::Confirming)
        {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    &format!(
                        "Cannot remove vote. Referendum status: {}",
                        subsquare_referendum.state.status
                    ),
                )
                .await?;
            return Ok(());
        }
        self.telegram_client
            .send_message(
                chat_id,
                Some(thread_id),
                "⚙️ Removing the on-chain vote. Please give me some time.",
            )
            .await?;
        let remove_vote_result = self.voter.remove_vote(&chain, db_referendum.index).await?;
        self.postgres
            .set_referendum_last_vote_id(db_referendum.id, None)
            .await?;
        self.postgres.remove_vote(last_vote_id).await?;
        let message = format!(
            "Removed on-chain vote.\nhttps://{}.subscan.io/extrinsic/{}-{}",
            chain.chain.to_lowercase(),
            remove_vote_result.0,
            remove_vote_result.1
        );
        self.opensquare_client
            .make_appendant_on_proposal(&chain, &opensquare_referendum.id, &message)
            .await?;
        self.telegram_client
            .send_message(chat_id, Some(thread_id), &message)
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

    pub(crate) async fn process_force_vote_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        username: &str,
        vote: Option<bool>,
    ) -> anyhow::Result<()> {
        if !CONFIG.voter.voting_admin_usernames.contains(username) {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    "This command can only be called by a voting admin.",
                )
                .await?;
            return Ok(());
        }
        let thread_id = if let Some(thread_id) = thread_id {
            thread_id
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    "This command can only be called from a referendum topic.",
                )
                .await?;
            return Ok(());
        };
        let db_referendum = if let Some(referendum) = self
            .postgres
            .get_referendum_by_telegram_chat_and_thread_id(chat_id, thread_id)
            .await?
        {
            if referendum.is_terminated {
                self.telegram_client
                    .send_message(
                        chat_id,
                        Some(thread_id),
                        "Referendum has been terminated. Cannot remove vote.",
                    )
                    .await?;
                return Ok(());
            }
            referendum
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum not found in the storage. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let chain = Chain::from_id(db_referendum.network_id);
        let cid = if let Some(cid) = &db_referendum.opensquare_cid {
            cid
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "OpenSquare CID not found in the referendum record. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let opensquare_referendum =
            if let Some(referendum) = self.opensquare_client.fetch_referendum(cid).await? {
                referendum
            } else {
                self.telegram_client
                    .send_message(
                        chat_id,
                        Some(thread_id),
                        "Referendum not found on OpenSquare. Contact admin.",
                    )
                    .await?;
                return Ok(());
            };
        let subsquare_referendum = if let Some(referendum) = self
            .subsquare_client
            .fetch_referendum(&chain, db_referendum.index)
            .await?
        {
            referendum
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum not found on SubSquare. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        if !(subsquare_referendum.state.status == ReferendumStatus::Deciding
            || subsquare_referendum.state.status == ReferendumStatus::Preparing
            || subsquare_referendum.state.status == ReferendumStatus::Confirming)
        {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    &format!(
                        "Cannot vote. Referendum status: {}",
                        subsquare_referendum.state.status
                    ),
                )
                .await?;
            return Ok(());
        }
        self.telegram_client
            .send_message(
                chat_id,
                Some(thread_id),
                "⚙️ Preparing the on-chain submission. Please give me some time.",
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
            )
            .await?;
        self.postgres
            .set_referendum_last_vote_id(db_referendum.id, Some(vote_id as u32))
            .await?;
        let message = format!(
            "Force-voted {}.\nhttps://{}.subscan.io/extrinsic/{}-{}",
            (if let Some(vote) = vote {
                if vote {
                    "aye"
                } else {
                    "nay"
                }
            } else {
                "abstain"
            })
            .to_string()
            .to_uppercase(),
            chain.chain.to_lowercase(),
            block_number,
            extrinsic_index,
        );
        self.opensquare_client
            .make_appendant_on_proposal(&chain, &opensquare_referendum.id, &message)
            .await?;
        self.telegram_client
            .send_message(chat_id, Some(thread_id), &message)
            .await?;
        Ok(())
    }

    pub(crate) async fn process_vote_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        username: &str,
        post_feedback: bool,
    ) -> anyhow::Result<()> {
        if !CONFIG.voter.voting_admin_usernames.contains(username) {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    "This command can only be called by a voting admin.",
                )
                .await?;
            return Ok(());
        }
        let thread_id = if let Some(thread_id) = thread_id {
            thread_id
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    "This command can only be called from a referendum topic.",
                )
                .await?;
            return Ok(());
        };
        let db_referendum = if let Some(referendum) = self
            .postgres
            .get_referendum_by_telegram_chat_and_thread_id(chat_id, thread_id)
            .await?
        {
            if referendum.is_terminated {
                self.telegram_client
                    .send_message(
                        chat_id,
                        Some(thread_id),
                        "Referendum has been terminated. Cannot remove vote.",
                    )
                    .await?;
                return Ok(());
            }
            referendum
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum not found in the storage. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let cid = if let Some(cid) = &db_referendum.opensquare_cid {
            cid
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "OpenSquare CID not found in the referendum record. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let chain = Chain::from_id(db_referendum.network_id);
        let subsquare_referendum = if let Some(referendum) = self
            .subsquare_client
            .fetch_referendum(&chain, db_referendum.index)
            .await?
        {
            referendum
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum not found on SubSquare. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        if !(subsquare_referendum.state.status == ReferendumStatus::Deciding
            || subsquare_referendum.state.status == ReferendumStatus::Preparing
            || subsquare_referendum.state.status == ReferendumStatus::Confirming)
        {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    &format!(
                        "Cannot vote. Referendum status: {}",
                        subsquare_referendum.state.status
                    ),
                )
                .await?;
            return Ok(());
        }
        let opensquare_referendum =
            if let Some(referendum) = self.opensquare_client.fetch_referendum(cid).await? {
                referendum
            } else {
                self.telegram_client
                    .send_message(
                        chat_id,
                        Some(thread_id),
                        "Referendum not found on OpenSquare. Contact admin.",
                    )
                    .await?;
                return Ok(());
            };
        let opensquare_votes = if let Some(opensquare_votes) =
            self.opensquare_client.fetch_referendum_votes(cid).await?
        {
            opensquare_votes
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum not found on OpenSquare by CID. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let voting_policy = if let Some(voting_policy) =
            VotingPolicy::voting_policy_for_track(db_referendum.track)
        {
            voting_policy
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    &format!(
                        "No voting policy is defined for {}.",
                        db_referendum.track.name(),
                    ),
                )
                .await?;
            return Ok(());
        };
        let mut aye_count = 0;
        let mut nay_count = 0;
        let mut abstain_count = 0;
        for vote in opensquare_votes.iter() {
            if vote.choices.contains(&OpenSquareVote::Aye) {
                aye_count += 1;
            } else if vote.choices.contains(&OpenSquareVote::Nay) {
                nay_count += 1;
            } else if vote.choices.contains(&OpenSquareVote::Abstain) {
                abstain_count += 1;
            }
        }
        let participation = aye_count + nay_count + abstain_count;
        let participation_percent = (participation * 100) / CONFIG.voter.member_count;
        let quorum_percent = (aye_count * 100) / CONFIG.voter.member_count;
        let aye_percent = if (aye_count + nay_count) == 0 {
            0
        } else {
            (aye_count * 100) / (aye_count + nay_count)
        };
        let past_votes = self.postgres.get_referendum_votes(db_referendum.id).await?;
        let vote: Option<bool>;
        let mut message = format!("🟢 {aye_count} • 🔴 {nay_count} • ⚪️ {abstain_count}");
        if participation_percent < voting_policy.participation_percent as u32 {
            vote = None;
            message = format!(
                "{message}\n{}% participation not met.\n**Vote #{}: ABSTAIN**",
                voting_policy.participation_percent,
                past_votes.len() + 1,
            );
        } else if quorum_percent < voting_policy.quorum_percent as u32 {
            vote = Some(false);
            message = format!(
                "{message}\n{}% quorum not met.\n**Vote #{}: NAY**",
                voting_policy.quorum_percent,
                past_votes.len() + 1,
            );
        } else if aye_percent <= voting_policy.majority_percent as u32 {
            vote = Some(false);
            message = format!(
                "{message}\n{}% majority not met.\n**Voted #{}: NAY**",
                voting_policy.majority_percent,
                past_votes.len() + 1,
            );
        } else {
            vote = Some(true);
            message = format!("{message}\n**Vote #{}: AYE**", past_votes.len() + 1,)
        };
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
                vote,
                balance,
                conviction,
            )
            .await?;
        let (subsquare_cid, subsquare_index) = if post_feedback {
            log::info!("Get OpenAI feedback summary.");
            let feedback = self
                .openai_client
                .fetch_feedback_summary(&opensquare_votes)
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
                        cid,
                        &first_vote_cid,
                        &db_referendum.track,
                        &voting_policy,
                        past_votes.len() as u32 + 1,
                        (aye_count, nay_count, abstain_count),
                        CONFIG.voter.member_count,
                        vote,
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
                        cid,
                        &db_referendum.track,
                        &voting_policy,
                        past_votes.len() as u32 + 1,
                        (aye_count, nay_count, abstain_count),
                        CONFIG.voter.member_count,
                        vote,
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
                vote,
                balance,
                conviction,
                subsquare_cid.as_deref(),
                subsquare_index,
            )
            .await?;
        self.postgres
            .set_referendum_last_vote_id(db_referendum.id, Some(vote_id as u32))
            .await?;
        message = format!(
            "{message}\nhttps://{}.subscan.io/extrinsic/{}-{}",
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
            .make_appendant_on_proposal(&chain, cid, &message)
            .await?;
        Ok(())
    }

    pub(crate) async fn process_notify_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        username: &str,
    ) -> anyhow::Result<()> {
        if !CONFIG.voter.voting_admin_usernames.contains(username) {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    "This command can only be called by a voting admin.",
                )
                .await?;
            return Ok(());
        }
        let thread_id = if let Some(thread_id) = thread_id {
            thread_id
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    "This command can only be called from a referendum topic.",
                )
                .await?;
            return Ok(());
        };
        let db_referendum = if let Some(referendum) = self
            .postgres
            .get_referendum_by_telegram_chat_and_thread_id(chat_id, thread_id)
            .await?
        {
            if referendum.is_terminated {
                self.telegram_client
                    .send_message(
                        chat_id,
                        Some(thread_id),
                        "Referendum has been terminated. Cannot remove vote.",
                    )
                    .await?;
                return Ok(());
            }
            referendum
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum not found in the storage. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let cid = if let Some(cid) = &db_referendum.opensquare_cid {
            cid
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "OpenSquare CID not found in the referendum record. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let opensquare_votes = if let Some(opensquare_votes) =
            self.opensquare_client.fetch_referendum_votes(cid).await?
        {
            opensquare_votes
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum not found on OpenSquare by CID. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let voted_members: Vec<AccountId> = opensquare_votes.iter().map(|v| v.voter).collect();
        let non_voted_member_telegram_usernames: Vec<String> = self
            .postgres
            .get_all_members()
            .await?
            .iter()
            .filter(|m| !voted_members.contains(&m.polkadot_address))
            .map(|m| format!("@{}", m.telegram_username))
            .collect();
        let message = if non_voted_member_telegram_usernames.is_empty() {
            "All members have voted.".to_string()
        } else {
            format!(
                "🔔 {} please vote!",
                non_voted_member_telegram_usernames
                    .join(", ")
                    .replace("_", "\\_"),
            )
        };
        self.telegram_client
            .send_message(chat_id, Some(thread_id), &message)
            .await?;
        Ok(())
    }

    pub(crate) async fn process_archive_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        username: &str,
    ) -> anyhow::Result<()> {
        if !CONFIG.voter.voting_admin_usernames.contains(username) {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    "This command can only be called by a voting admin.",
                )
                .await?;
            return Ok(());
        }
        let thread_id = if let Some(thread_id) = thread_id {
            thread_id
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    "This command can only be called from a referendum topic.",
                )
                .await?;
            return Ok(());
        };
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

    pub(crate) async fn process_coi_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        has_coi: bool,
        username: &str,
    ) -> anyhow::Result<()> {
        if !CONFIG.voter.voting_admin_usernames.contains(username) {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    "This command can only be called by a voting admin.",
                )
                .await?;
            return Ok(());
        }
        let thread_id = if let Some(thread_id) = thread_id {
            thread_id
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    "This command can only be called from a referendum topic.",
                )
                .await?;
            return Ok(());
        };
        let db_referendum = if let Some(referendum) = self
            .postgres
            .get_referendum_by_telegram_chat_and_thread_id(chat_id, thread_id)
            .await?
        {
            if referendum.is_terminated {
                self.telegram_client
                    .send_message(
                        chat_id,
                        Some(thread_id),
                        "Referendum has been terminated. Cannot remove vote.",
                    )
                    .await?;
                return Ok(());
            }
            referendum
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum not found in the storage. Contact admin.",
                )
                .await?;
            return Ok(());
        };
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
                    "Referendum has not marked for conlict of interest.",
                )
                .await?;
            return Ok(());
        }
        let chain = Chain::from_id(db_referendum.network_id);
        let subsquare_referendum = if let Some(referendum) = self
            .subsquare_client
            .fetch_referendum(&chain, db_referendum.index)
            .await?
        {
            referendum
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "Referendum not found on SubSquare. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        if !(subsquare_referendum.state.status == ReferendumStatus::Deciding
            || subsquare_referendum.state.status == ReferendumStatus::Preparing
            || subsquare_referendum.state.status == ReferendumStatus::Confirming)
        {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    &format!(
                        "Cannot report or remove CoI. Referendum status: {}",
                        subsquare_referendum.state.status
                    ),
                )
                .await?;
            return Ok(());
        }
        let cid = if let Some(cid) = &db_referendum.opensquare_cid {
            cid
        } else {
            self.telegram_client
                .send_message(
                    chat_id,
                    Some(thread_id),
                    "OpenSquare CID not found in the referendum record. Contact admin.",
                )
                .await?;
            return Ok(());
        };
        let opensquare_referendum =
            if let Some(referendum) = self.opensquare_client.fetch_referendum(cid).await? {
                referendum
            } else {
                self.telegram_client
                    .send_message(
                        chat_id,
                        Some(thread_id),
                        "Referendum not found on OpenSquare. Contact admin.",
                    )
                    .await?;
                return Ok(());
            };
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
