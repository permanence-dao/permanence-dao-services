use frankenstein::{
    client_reqwest::Bot, AsyncTelegramApi, ChatId, CreateForumTopicParams, EditForumTopicParams,
    GetUpdatesParams, LinkPreviewOptions, Message, MethodResponse, ParseMode, SendMessageParams,
    Update,
};
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
                frankenstein::AllowedUpdate::ChatMember,
                frankenstein::AllowedUpdate::MyChatMember,
                frankenstein::AllowedUpdate::Message,
                frankenstein::AllowedUpdate::CallbackQuery,
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
            "Create Telegram topic for {} referendum ${}.",
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
                    "[{}] {} #{} - {}",
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
            title,
            referendum.state.status,
        );
        let message = if let Some(content_summary) = &referendum.content_summary {
            if let Some(summary) = &content_summary.summary {
                format!("{}\n\n**AI Summary:**\n{}", message, summary)
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

    pub async fn update_referendum_topic_name(
        &self,
        chat_id: i64,
        thread_id: i32,
        name: &str,
        maybe_status_text: Option<&str>,
        status_emoji: &str,
    ) -> anyhow::Result<bool> {
        let stickers = self.telegram_api.get_forum_topic_icon_stickers().await?;
        let mut checkmark_emoji_id = None;
        for sticker in stickers.result.iter() {
            if sticker.emoji == Some(status_emoji.to_string()) {
                checkmark_emoji_id = sticker.custom_emoji_id.clone();
            }
        }

        let params = EditForumTopicParams::builder()
            .chat_id(ChatId::Integer(chat_id))
            .message_thread_id(thread_id)
            .maybe_icon_custom_emoji_id(checkmark_emoji_id)
            .name(if let Some(status_text) = maybe_status_text {
                format!("[{status_text}] {name}")
            } else {
                name.to_string()
            })
            .build();
        let result = self.telegram_api.edit_forum_topic(&params).await?;
        Ok(result.result)
    }
}
