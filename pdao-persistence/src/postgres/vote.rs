use crate::postgres::PostgreSQLStorage;
use pdao_types::governance::Vote;

type VoteRecord = (
    i32,
    i32,
    i32,
    i32,
    String,
    i64,
    i32,
    Option<bool>,
    String,
    i32,
    bool,
    Option<String>,
    Option<i32>,
    bool,
    bool,
);

fn vote_record_into_vote(record: &VoteRecord) -> anyhow::Result<Vote> {
    Ok(Vote {
        id: record.0 as u32,
        network_id: record.1 as u32,
        referendum_id: record.2 as u32,
        index: record.3 as u32,
        block_hash: record.4.clone(),
        block_number: record.5 as u64,
        extrinsic_index: record.6 as u32,
        vote: record.7,
        balance: record.8.parse()?,
        conviction: record.9 as u32,
        is_removed: record.10,
        subsquare_comment_cid: record.11.clone(),
        subsquare_comment_index: record.12.map(|i| i as u32),
        has_coi: record.13,
        is_forced: record.14,
    })
}

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
        has_coi: bool,
        is_forced: bool,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO pdao_vote (network_id, referendum_id, index, block_hash, block_number, extrinsic_index, vote, balance, conviction, subsquare_comment_cid, subsquare_comment_index, has_coi, is_forced)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
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
            .bind(has_coi)
            .bind(is_forced)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0)
    }

    pub async fn set_vote_removed(&self, vote_id: u32) -> anyhow::Result<Option<i32>> {
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            UPDATE pdao_vote SET is_removed = true
            WHERE id = $1
            RETURNING id
            "#,
        )
        .bind(vote_id as i32)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_result.map(|r| r.0))
    }

    pub async fn get_referendum_votes(&self, referendum_id: u32) -> anyhow::Result<Vec<Vote>> {
        let db_votes: Vec<VoteRecord> = sqlx::query_as(
            r#"
            SELECT id, network_id, referendum_id, index, block_hash, block_number, extrinsic_index, vote, balance, conviction, is_removed, subsquare_comment_cid, subsquare_comment_index, has_coi, is_forced
            FROM pdao_vote
            WHERE referendum_id = $1
            ORDER BY id ASC
            "#,
        )
            .bind(referendum_id as i32)
            .fetch_all(&self.connection_pool)
            .await?;
        let mut votes = Vec::new();
        for db_vote in db_votes.iter() {
            votes.push(vote_record_into_vote(db_vote)?);
        }
        Ok(votes)
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

    #[allow(clippy::too_many_arguments)]
    pub async fn save_member_vote(
        &self,
        vote_id: u32,
        cid: &str,
        network_id: u32,
        referendum_id: u32,
        referendum_index: u32,
        address: &str,
        feedback: &str,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO pdao_member_vote (vote_id, cid, network_id, referendum_id, index, address, feedback)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
        )
            .bind(vote_id as i32)
            .bind(cid)
            .bind(network_id as i32)
            .bind(referendum_id as i32)
            .bind(referendum_index as i32)
            .bind(address)
            .bind(feedback)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0)
    }
}
