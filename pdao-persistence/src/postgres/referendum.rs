use crate::postgres::PostgreSQLStorage;
use pdao_types::governance::subsquare::SubSquareReferendum as OpensquareReferendum;
use pdao_types::governance::track::Track;
use pdao_types::governance::{Referendum, ReferendumStatus};
use std::str::FromStr;

type ReferendumRecord = (
    i32,
    i32,
    i32,
    i32,
    String,
    Option<String>,
    Option<String>,
    String,
    i64,
    i32,
    i32,
    Option<String>,
    Option<String>,
    Option<i32>,
);

fn referendum_record_into_referendum(record: &ReferendumRecord) -> anyhow::Result<Referendum> {
    Ok(Referendum {
        id: record.0 as u32,
        network_id: record.1 as u32,
        track: Track::from_id(record.2 as u16).unwrap(),
        index: record.3 as u32,
        status: ReferendumStatus::from_str(&record.4)?,
        title: record.5.clone(),
        content: record.6.clone(),
        content_type: record.7.clone(),
        telegram_chat_id: record.8,
        telegram_topic_id: record.9,
        telegram_intro_message_id: record.10,
        opensquare_cid: record.11.clone(),
        opensquare_post_uid: record.12.clone(),
        last_vote_id: record.13.map(|id| id as u32),
    })
}

impl PostgreSQLStorage {
    pub async fn save_referendum(
        &self,
        network_id: u32,
        referendum: &OpensquareReferendum,
        opensquare_cid: &str,
        opensquare_post_uid: &str,
        telegram_chat_id: i64,
        new_telegram_topic_response: (i32, i32),
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO pdao_referendum (network_id, track_id, index, status, title, content, content_type, telegram_chat_id, telegram_topic_id, telegram_intro_message_id, opensquare_cid, opensquare_post_uid)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT(network_id, index) DO UPDATE
            SET track_id = EXCLUDED.track_id, title = EXCLUDED.title, content = EXCLUDED.content, content_type = EXCLUDED.content_type, telegram_chat_id = EXCLUDED.telegram_chat_id, telegram_topic_id = EXCLUDED.telegram_topic_id, telegram_intro_message_id = EXCLUDED.telegram_intro_message_id, opensquare_cid = EXCLUDED.opensquare_cid, opensquare_post_uid = EXCLUDED.opensquare_post_uid
            RETURNING id
            "#,
        )
            .bind(network_id as i32)
            .bind(referendum.track_id as i32)
            .bind(referendum.referendum_index as i32)
            .bind(referendum.state.status.to_string())
            .bind(referendum.title.clone())
            .bind(referendum.content.clone())
            .bind(referendum.content_type.clone())
            .bind(telegram_chat_id)
            .bind(new_telegram_topic_response.0)
            .bind(new_telegram_topic_response.1)
            .bind(opensquare_cid)
            .bind(opensquare_post_uid)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0)
    }

    pub async fn get_referendum_by_index(
        &self,
        network_id: u32,
        referendum_index: u32,
    ) -> anyhow::Result<Option<Referendum>> {
        let maybe_record: Option<ReferendumRecord> = sqlx::query_as(
            r#"
            SELECT id, network_id, track_id, index, status, title, content, content_type, telegram_chat_id, telegram_topic_id, telegram_intro_message_id, opensquare_cid, opensquare_post_uid, last_vote_id
            FROM pdao_referendum
            WHERE network_id = $1 AND index = $2
            "#,
        )
            .bind(network_id as i32)
            .bind(referendum_index as i32)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(record) = maybe_record {
            Ok(Some(referendum_record_into_referendum(&record)?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_referendum_by_telegram_chat_and_thread_id(
        &self,
        chat_id: i64,
        thread_id: i32,
    ) -> anyhow::Result<Option<Referendum>> {
        let maybe_record: Option<ReferendumRecord> = sqlx::query_as(
            r#"
            SELECT id, network_id, track_id, index, status, title, content, content_type, telegram_chat_id, telegram_topic_id, telegram_intro_message_id, opensquare_cid, opensquare_post_uid, last_vote_id
            FROM pdao_referendum
            WHERE telegram_chat_id = $1 AND telegram_topic_id = $2
            "#,
        )
            .bind(chat_id)
            .bind(thread_id)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(record) = maybe_record {
            Ok(Some(referendum_record_into_referendum(&record)?))
        } else {
            Ok(None)
        }
    }
}
