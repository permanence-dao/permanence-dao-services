use crate::command::util::require_member;
use crate::TelegramBot;

impl TelegramBot {
    pub(crate) async fn process_mark_leave_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        username: &str,
    ) -> anyhow::Result<()> {
        let member = require_member(&self.postgres, username).await?;
        if member.is_on_leave {
            return Err(anyhow::Error::msg(format!(
                "You are already on leave, @{username}."
            )));
        }
        self.postgres.mark_member_leave(member.id).await?;
        self.telegram_client
            .send_message(
                chat_id,
                thread_id,
                &format!("Happy holidays, @{username}! See you soon."),
            )
            .await?;
        Ok(())
    }
}
