use frankenstein::methods::{
    CreateForumTopicParams, DeleteForumTopicParams, EditForumTopicParams, GetUpdatesParams,
    SendDocumentParams, SendMessageParams,
};
use frankenstein::response::MethodResponse;
use frankenstein::types::{AllowedUpdate, ChatId, LinkPreviewOptions, Message};
use frankenstein::updates::Update;
use frankenstein::{client_reqwest::Bot, AsyncTelegramApi, ParseMode};
use pdao_config::Config;
use pdao_types::governance::opensquare::OpenSquareNewProposalResponse;
use pdao_types::governance::subsquare::SubSquareReferendum;
use pdao_types::governance::track::Track;
use pdao_types::substrate::chain::Chain;

pub struct TelegramClient {
    telegram_api: Bot,
}

impl TelegramClient {
    pub fn new(config: &Config) -> Self {
        Self {
            telegram_api: Bot::new(&config.telegram.api_token),
        }
    }

    pub async fn get_updates(&self, offset: Option<i64>) -> anyhow::Result<Vec<Update>> {
        let params = GetUpdatesParams {
            offset,
            limit: None,
            timeout: None,
            allowed_updates: Some(vec![
                AllowedUpdate::ChatMember,
                AllowedUpdate::MyChatMember,
                AllowedUpdate::Message,
                AllowedUpdate::CallbackQuery,
            ]),
        };
        let result = self.telegram_api.get_updates(&params).await?;
        Ok(result.result)
    }

    pub async fn create_referendum_topic(
        &self,
        chain: &Chain,
        config: &Config,
        referendum: &SubSquareReferendum,
        new_opensquare_proposal_response: &OpenSquareNewProposalResponse,
    ) -> anyhow::Result<(i32, i32)> {
        log::info!(
            "Create Telegram topic for {} referendum {}.",
            chain.token_ticker,
            referendum.referendum_index
        );
        let stickers = self.telegram_api.get_forum_topic_icon_stickers().await?;
        let mut ballot_emoji_id = None;
        for sticker in stickers.result.iter() {
            if sticker.emoji == Some("🗳".to_string()) {
                ballot_emoji_id = sticker.custom_emoji_id.clone();
            }
        }
        let track = Track::from_id(referendum.track_id).unwrap();
        let title = if let Some(title) = &referendum.title {
            title
        } else {
            "N/A"
        };
        let create_topic_response = self
            .telegram_api
            .create_forum_topic(&CreateForumTopicParams {
                chat_id: ChatId::Integer(config.telegram.chat_id),
                name: format!(
                    "[V0] [{}] {} #{} - {}",
                    track.short_name(),
                    chain.token_ticker,
                    referendum.referendum_index,
                    title,
                ),
                icon_color: None,
                icon_custom_emoji_id: ballot_emoji_id.clone(),
            })
            .await?;
        let url = format!(
            "https://{}.subsquare.io/referenda/{}",
            chain.chain, referendum.referendum_index,
        );
        let message = format!(
            "• {} [#{}]({})\n• {}\n• {}\n• Status: {}",
            chain.display,
            referendum.referendum_index,
            url,
            track.name(),
            title.replace("_", "\\_"),
            referendum.state.status,
        );
        let message = if let Some(content_summary) = &referendum.content_summary {
            if let Some(summary) = &content_summary.summary {
                format!("{message}\n\n**AI Summary:**\n{}", summary.replace("_", "\\_"))
            } else {
                message
            }
        } else {
            message
        };
        let message = format!(
            "{}\n\n🗳️ Vote [here](https://voting.opensquare.io/space/{}/proposal/{}).",
            message,
            config.referendum_importer.opensquare_space,
            new_opensquare_proposal_response.cid,
        );
        let send_message_response = self
            .send_message(
                config.telegram.chat_id,
                Some(create_topic_response.result.message_thread_id),
                &message,
            )
            .await?;
        log::info!(
            "Created Telegram topic for {} referendum ${} with topic id {}.",
            chain.token_ticker,
            referendum.referendum_index,
            create_topic_response.result.message_thread_id,
        );
        Ok((
            create_topic_response.result.message_thread_id,
            send_message_response.result.message_id,
        ))
    }

    pub async fn send_message(
        &self,
        chat_id: i64,
        thread_id: Option<i32>,
        message: &str,
    ) -> anyhow::Result<MethodResponse<Message>> {
        let response = self
            .telegram_api
            .send_message(&SendMessageParams {
                business_connection_id: None,
                chat_id: ChatId::Integer(chat_id),
                message_thread_id: thread_id,
                text: html_escape::encode_text(message).to_string(),
                #[allow(deprecated)]
                parse_mode: Some(ParseMode::Markdown),
                entities: None,
                link_preview_options: Some(LinkPreviewOptions {
                    is_disabled: Some(true),
                    url: None,
                    prefer_small_media: None,
                    prefer_large_media: None,
                    show_above_text: None,
                }),
                disable_notification: None,
                protect_content: None,
                allow_paid_broadcast: None,
                message_effect_id: None,
                reply_parameters: None,
                reply_markup: None,
            })
            .await?;
        Ok(response)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_referendum_topic_name(
        &self,
        chat_id: i64,
        thread_id: i32,
        name: &str,
        has_coi: bool,
        maybe_status_text: Option<&str>,
        vote_count_status: &str,
        status_emoji: &str,
    ) -> anyhow::Result<bool> {
        let stickers = self.telegram_api.get_forum_topic_icon_stickers().await?;
        let mut checkmark_emoji_id = None;
        for sticker in stickers.result.iter() {
            if sticker.emoji == Some(status_emoji.to_string()) {
                checkmark_emoji_id = sticker.custom_emoji_id.clone();
            }
        }

        let coi_status = if has_coi { "[CoI] " } else { "" };
        let params = EditForumTopicParams::builder()
            .chat_id(ChatId::Integer(chat_id))
            .message_thread_id(thread_id)
            .maybe_icon_custom_emoji_id(checkmark_emoji_id)
            .name(if let Some(status_text) = maybe_status_text {
                format!("[{status_text}] [{vote_count_status}] {coi_status}{name}")
            } else {
                format!("[{vote_count_status}] {coi_status}{name}")
            })
            .build();
        let result = self.telegram_api.edit_forum_topic(&params).await?;
        Ok(result.result)
    }

    pub async fn delete_referendum_topic(
        &self,
        chat_id: i64,
        thread_id: i32,
    ) -> anyhow::Result<()> {
        let params = DeleteForumTopicParams::builder()
            .chat_id(chat_id)
            .message_thread_id(thread_id)
            .build();
        self.telegram_api.delete_forum_topic(&params).await?;
        Ok(())
    }

    pub async fn upload_file(
        &self,
        file_path: &str,
        chat_id: i64,
        thread_id: i32,
        caption: Option<&str>,
    ) -> anyhow::Result<()> {
        let file = std::path::PathBuf::from(file_path);
        let params = SendDocumentParams::builder()
            .chat_id(chat_id)
            .message_thread_id(thread_id)
            .document(file)
            .maybe_caption(caption)
            .build();
        self.telegram_api.send_document(&params).await?;
        Ok(())
    }

    pub async fn create_archive_topic(&self, config: &Config) -> anyhow::Result<i32> {
        log::info!("Create archive topic.");
        let stickers = self.telegram_api.get_forum_topic_icon_stickers().await?;
        let mut briefcase_emoji_id = None;
        for sticker in stickers.result.iter() {
            if sticker.emoji == Some("💼".to_string()) {
                briefcase_emoji_id = sticker.custom_emoji_id.clone();
            }
        }

        let create_topic_response = self
            .telegram_api
            .create_forum_topic(&CreateForumTopicParams {
                chat_id: ChatId::Integer(config.telegram.chat_id),
                name: "Archive".to_string(),
                icon_color: None,
                icon_custom_emoji_id: briefcase_emoji_id.clone(),
            })
            .await?;
        log::info!(
            "Created Telegram archive topic with thread id {}.",
            create_topic_response.result.message_thread_id,
        );
        Ok(create_topic_response.result.message_thread_id)
    }
}
