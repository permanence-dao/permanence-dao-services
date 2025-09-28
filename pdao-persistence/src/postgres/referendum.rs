use crate::postgres::PostgreSQLStorage;
use pdao_types::governance::subsquare::SubSquareReferendum as OpensquareReferendum;
use pdao_types::governance::track::Track;
use pdao_types::governance::{Referendum, ReferendumStatus};
use sqlx::FromRow;
use std::str::FromStr;

#[derive(Debug, FromRow)]
struct ReferendumRow {
    pub id: i32,
    pub network_id: i32,
    pub track_id: i32,
    pub index: i32,
    pub status: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub content_type: String,
    pub telegram_chat_id: i64,
    pub telegram_topic_id: i32,
    pub telegram_intro_message_id: i32,
    pub opensquare_cid: Option<String>,
    pub opensquare_post_uid: Option<String>,
    pub last_vote_id: Option<i32>,
    pub is_terminated: bool,
    pub has_coi: bool,
    pub is_archived: bool,
    pub preimage_exists: bool,
}

fn referendum_row_into_referendum(row: &ReferendumRow) -> anyhow::Result<Referendum> {
    Ok(Referendum {
        id: row.id as u32,
        network_id: row.network_id as u32,
        track: Track::from_id(row.track_id as u16).unwrap(),
        index: row.index as u32,
        status: ReferendumStatus::from_str(&row.status)?,
        title: row.title.clone(),
        content: row.content.clone(),
        content_type: row.content_type.clone(),
        telegram_chat_id: row.telegram_chat_id,
        telegram_topic_id: row.telegram_topic_id,
        telegram_intro_message_id: row.telegram_intro_message_id,
        opensquare_cid: row.opensquare_cid.clone(),
        opensquare_post_uid: row.opensquare_post_uid.clone(),
        last_vote_id: row.last_vote_id.map(|id| id as u32),
        is_terminated: row.is_terminated,
        has_coi: row.has_coi,
        is_archived: row.is_archived,
        preimage_exists: row.preimage_exists,
    })
}

impl PostgreSQLStorage {
    pub async fn save_referendum(
        &self,
        network_id: u32,
        referendum: &OpensquareReferendum,
        preimage_exists: bool,
        opensquare_cid: &str,
        opensquare_post_uid: &str,
        telegram_chat_id: i64,
        new_telegram_topic_response: (i32, i32),
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO pdao_referendum (network_id, track_id, index, status, title, content, content_type, telegram_chat_id, telegram_topic_id, telegram_intro_message_id, opensquare_cid, opensquare_post_uid, preimage_exists)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT(network_id, index) DO UPDATE
            SET track_id = EXCLUDED.track_id, title = EXCLUDED.title, content = EXCLUDED.content, content_type = EXCLUDED.content_type, telegram_chat_id = EXCLUDED.telegram_chat_id, telegram_topic_id = EXCLUDED.telegram_topic_id, telegram_intro_message_id = EXCLUDED.telegram_intro_message_id, opensquare_cid = EXCLUDED.opensquare_cid, opensquare_post_uid = EXCLUDED.opensquare_post_uid, preimage_exists = EXCLUDED.preimage_exists
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
            .bind(preimage_exists)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0)
    }

    pub async fn get_referendum_by_index(
        &self,
        network_id: u32,
        referendum_index: u32,
    ) -> anyhow::Result<Option<Referendum>> {
        let maybe_row: Option<ReferendumRow> = sqlx::query_as::<_, ReferendumRow>(
            r#"
            SELECT id, network_id, track_id, index, status, title, content, content_type, telegram_chat_id, telegram_topic_id, telegram_intro_message_id, opensquare_cid, opensquare_post_uid, last_vote_id, is_terminated, has_coi, is_archived, preimage_exists
            FROM pdao_referendum
            WHERE network_id = $1 AND index = $2
            "#,
        )
            .bind(network_id as i32)
            .bind(referendum_index as i32)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(row) = maybe_row {
            Ok(Some(referendum_row_into_referendum(&row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_referendum_by_telegram_chat_and_thread_id(
        &self,
        chat_id: i64,
        thread_id: i32,
    ) -> anyhow::Result<Option<Referendum>> {
        let maybe_row: Option<ReferendumRow> = sqlx::query_as::<_, ReferendumRow>(
            r#"
            SELECT id, network_id, track_id, index, status, title, content, content_type, telegram_chat_id, telegram_topic_id, telegram_intro_message_id, opensquare_cid, opensquare_post_uid, last_vote_id, is_terminated, has_coi, is_archived
            FROM pdao_referendum
            WHERE telegram_chat_id = $1 AND telegram_topic_id = $2
            "#,
        )
            .bind(chat_id)
            .bind(thread_id)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(row) = maybe_row {
            Ok(Some(referendum_row_into_referendum(&row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn terminate_referendum(&self, referendum_id: u32) -> anyhow::Result<Option<i32>> {
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            UPDATE pdao_referendum SET is_terminated = TRUE
            WHERE id = $1
            RETURNING id
            "#,
        )
        .bind(referendum_id as i32)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_result.map(|r| r.0))
    }

    pub async fn set_referendum_has_coi(
        &self,
        referendum_id: u32,
        has_coi: bool,
    ) -> anyhow::Result<Option<i32>> {
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            UPDATE pdao_referendum SET has_coi = $1
            WHERE id = $2
            RETURNING id
            "#,
        )
        .bind(has_coi)
        .bind(referendum_id as i32)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_result.map(|r| r.0))
    }

    pub async fn archive_referendum(
        &self,
        referendum_id: u32,
        message_archive: &str,
    ) -> anyhow::Result<Option<i32>> {
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            UPDATE pdao_referendum SET message_archive = $1, is_archived = true
            WHERE id = $2
            RETURNING id
            "#,
        )
        .bind(message_archive)
        .bind(referendum_id as i32)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_result.map(|r| r.0))
    }

    pub async fn update_referendum_status(
        &self,
        referendum_id: u32,
        referendum_status: &ReferendumStatus,
    ) -> anyhow::Result<Option<i32>> {
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            UPDATE pdao_referendum SET status = $1
            WHERE id = $2
            RETURNING id
            "#,
        )
        .bind(referendum_status.to_string())
        .bind(referendum_id as i32)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_result.map(|r| r.0))
    }

    pub async fn set_referendum_preimage_exists(
        &self,
        referendum_id: u32,
        preimage_exists: bool,
    ) -> anyhow::Result<Option<i32>> {
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            UPDATE pdao_referendum SET preimage_exists = $1
            WHERE id = $2
            RETURNING id
            "#,
        )
        .bind(preimage_exists)
        .bind(referendum_id as i32)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_result.map(|r| r.0))
    }
}
