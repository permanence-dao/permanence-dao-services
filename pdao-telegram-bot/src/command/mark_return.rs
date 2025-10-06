use crate::command::util::require_member;
use crate::TelegramBot;

impl TelegramBot {
    pub(crate) async fn process_mark_return_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        username: &str,
    ) -> anyhow::Result<()> {
        let member = require_member(&self.postgres, username).await?;
        if !member.is_on_leave {
            self.telegram_client
                .send_message(
                    chat_id,
                    thread_id,
                    &format!("You are not on leave, @{username}."),
                    true,
                )
                .await?;
            return Ok(());
        }
        self.postgres.mark_member_return(member.id).await?;
        self.telegram_client
            .send_message(
                chat_id,
                thread_id,
                &format!("Welcome back, @{username}!"),
                true,
            )
            .await?;
        Ok(())
    }
}
