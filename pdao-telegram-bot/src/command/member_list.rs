use crate::TelegramBot;
use pdao_types::{Member, MembershipType};

impl TelegramBot {
    pub(crate) async fn process_member_list_command(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
    ) -> anyhow::Result<()> {
        fn get_member_list(members: &[Member], membership_type: MembershipType) -> String {
            if members.is_empty() {
                return "N/A".to_string();
            }
            members
                .iter()
                .enumerate()
                .filter(|m| m.1.membership_type == membership_type)
                .map(|m| {
                    format!(
                        "{}. {} {}",
                        m.0 + 1,
                        if m.1.is_on_leave { "🟡" } else { "🟢" },
                        m.1.name,
                    )
                })
                .collect::<Vec<String>>()
                .join("\n")
        }

        let mut members = self.postgres.get_all_members(true).await?;
        members.sort_by_key(|m| m.name.clone());
        let core_members = get_member_list(&members, MembershipType::Core);
        let community_members = get_member_list(&members, MembershipType::Community);
        let message = format!(
            "**CORE MEMBERS:**\n{core_members}\n\n**COMMUNITY MEMBERS:**\n{community_members}",
        );
        self.telegram_client
            .send_message(chat_id, thread_id, &message)
            .await?;
        Ok(())
    }
}
