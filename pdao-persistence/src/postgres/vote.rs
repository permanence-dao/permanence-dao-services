use crate::postgres::PostgreSQLStorage;

impl PostgreSQLStorage {
    #[allow(clippy::too_many_arguments)]
    pub async fn save_vote(
        &self,
        network_id: u32,
        referendum_id: u32,
        referendum_index: u32,
        block_hash: &str,
        block_number: u64,
        extrinsic_index: u32,
        vote: Option<bool>,
        balance: u128,
        conviction: u8,
        subsquare_comment_cid: Option<&str>,
        subsquare_comment_index: Option<u32>,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO pdao_vote (network_id, referendum_id, index, block_hash, block_number, extrinsic_index, vote, balance, conviction, subsquare_comment_cid, subsquare_comment_index)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id
            "#,
        )
            .bind(network_id as i32)
            .bind(referendum_id as i32)
            .bind(referendum_index as i32)
            .bind(block_hash)
            .bind(block_number as i64)
            .bind(extrinsic_index as i32)
            .bind(vote)
            .bind(balance.to_string())
            .bind(conviction as i32)
            .bind(subsquare_comment_cid)
            .bind(subsquare_comment_index.map(|index| index as i32))
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0)
    }

    pub async fn remove_vote(&self, vote_id: u32) -> anyhow::Result<Option<i32>> {
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            UPDATE pda_vote SET is_removed = true
            WHERE id = $1
            RETURNING id
            "#,
        )
        .bind(vote_id as i32)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_result.map(|r| r.0))
    }

    pub async fn get_referendum_vote_count(&self, referendum_id: u32) -> anyhow::Result<u32> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id) FROM pdao_vote
            WHERE referendum_id = $1
            "#,
        )
        .bind(referendum_id as i32)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 as u32)
    }

    pub async fn set_referendum_last_vote_id(
        &self,
        referendum_id: u32,
        vote_id: Option<u32>,
    ) -> anyhow::Result<Option<i32>> {
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            UPDATE pdao_referendum SET last_vote_id = $1
            WHERE id = $2
            RETURNING id
            "#,
        )
        .bind(vote_id.map(|id| id as i32))
        .bind(referendum_id as i32)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_result.map(|r| r.0))
    }
}
