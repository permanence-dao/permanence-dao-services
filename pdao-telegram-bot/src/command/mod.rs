use crate::{TelegramBot, CONFIG};
use pdao_referendum_importer::ReferendumImportError;
use pdao_types::governance::opensquare::OpenSquareVote;
use pdao_types::governance::policy::VotingPolicy;
use pdao_types::governance::ReferendumStatus;
use pdao_types::substrate::chain::Chain;
use std::str::FromStr;

impl TelegramBot {
    pub(crate) async fn process_import_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        args: &[String],
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
            .import_referendum(&chain, index)
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
        if opensquare_referendum.status.to_lowercase() != "active" {
            self.telegram_client
                .send_message(chat_id, Some(thread_id), "Referendum is not active.")
                .await?;
            return Ok(());
        }
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
        let aye_percent = if participation == 0 {
            0
        } else {
            (aye_count * 100) / participation
        };
        message = if participation_percent < voting_policy.participation_percent as u32 {
            format!(
                "{message}\n{}% participation not met.\nABSTAIN",
                voting_policy.participation_percent,
            )
        } else if aye_percent < voting_policy.quorum_percent as u32 {
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
        self.telegram_client
            .send_message(chat_id, Some(thread_id), &message)
            .await?;
        Ok(())
    }
}
