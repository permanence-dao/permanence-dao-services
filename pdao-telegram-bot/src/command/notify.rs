use crate::command::util::{
    require_db_referendum, require_db_referendum_is_active, require_opensquare_votes,
    require_thread, require_voting_admin,
};
use crate::TelegramBot;
use pdao_types::substrate::account_id::AccountId;
use pdao_types::substrate::chain::Chain;

impl TelegramBot {
    pub(crate) async fn process_notify_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        username: &str,
    ) -> anyhow::Result<()> {
        require_voting_admin(username)?;
        let thread_id = require_thread(thread_id)?;
        let db_referendum = require_db_referendum(&self.postgres, chat_id, thread_id).await?;
        require_db_referendum_is_active(&db_referendum)?;
        let member_account_ids = self
            .postgres
            .get_all_member_account_ids_for_chain(true, Chain::polkadot().id)
            .await?;
        let opensquare_votes = require_opensquare_votes(
            &self.opensquare_client,
            &db_referendum.opensquare_cid,
            &member_account_ids,
        )
        .await?;
        let voted_members: Vec<AccountId> = opensquare_votes.iter().map(|v| v.voter).collect();
        let non_voted_member_telegram_usernames: Vec<String> = self
            .postgres
            .get_all_members(false)
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
            .send_message(chat_id, Some(thread_id), &message, true)
            .await?;
        Ok(())
    }
}
