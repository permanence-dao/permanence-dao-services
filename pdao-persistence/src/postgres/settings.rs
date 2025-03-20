use crate::postgres::PostgreSQLStorage;

const ARCHIVE_THREAD_ID_KEY: &str = "archive_thread_id";

impl PostgreSQLStorage {
    pub async fn get_archive_thread_id(&self) -> anyhow::Result<Option<i32>> {
        let maybe_value: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT value
            FROM pdao_settings
            WHERE key = $1
            "#,
        )
        .bind(ARCHIVE_THREAD_ID_KEY)
        .fetch_optional(&self.connection_pool)
        .await?;
        if let Some(value) = maybe_value {
            return Ok(Some(value.0.parse::<i32>()?));
        }
        Ok(None)
    }

    pub async fn set_archive_thread_id(&self, archive_thread_id: i32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO pdao_settings (key, value)
            VALUES ($1, $2)
            ON CONFLICT(key) DO UPDATE
            SET value = EXCLUDED.value
            RETURNING key
            "#,
        )
        .bind(ARCHIVE_THREAD_ID_KEY)
        .bind(archive_thread_id.to_string())
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }
}
