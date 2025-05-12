use crate::command::util::require_member;
use crate::TelegramBot;

impl TelegramBot {
    pub(crate) async fn process_mark_return_command(&self, username: &str) -> anyhow::Result<()> {
        let member = require_member(&self.postgres, username).await?;
        if member.is_on_leave {
            return Err(anyhow::Error::msg(format!(
                "@{username} is already active."
            )));
        }
        self.postgres.mark_member_return(member.id).await?;
        Ok(())
    }
}
