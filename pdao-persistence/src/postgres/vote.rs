use crate::postgres::PostgreSQLStorage;
use pdao_types::governance::{MemberVote, PendingMemberVote, Vote};
use pdao_types::substrate::account_id::AccountId;
use std::str::FromStr;

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

type MemberVoteRecord = (
    i32,
    i32,
    String,
    i32,
    i32,
    i32,
    String,
    Option<bool>,
    String,
);

type PendingMemberVoteRecord = (i32, String, i32, i32, i32, String, Option<bool>, String);

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

fn member_vote_record_into_member_vote(record: &MemberVoteRecord) -> anyhow::Result<MemberVote> {
    Ok(MemberVote {
        id: record.0 as u32,
        vote_id: record.1 as u32,
        cid: record.2.to_string(),
        network_id: record.3 as u32,
        referendum_id: record.4 as u32,
        index: record.5 as u32,
        address: AccountId::from_str(&record.6)?,
        vote: record.7,
        feedback: record.8.clone(),
    })
}

fn pending_member_vote_record_into_pending_member_vote(
    record: &PendingMemberVoteRecord,
) -> anyhow::Result<PendingMemberVote> {
    Ok(PendingMemberVote {
        id: record.0 as u32,
        cid: record.1.to_string(),
        network_id: record.2 as u32,
        referendum_id: record.3 as u32,
        index: record.4 as u32,
        address: AccountId::from_str(&record.5)?,
        vote: record.6,
        feedback: record.7.clone(),
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

    pub async fn get_referendum_last_vote(
        &self,
        referendum_id: u32,
    ) -> anyhow::Result<Option<Vote>> {
        let db_vote: Option<VoteRecord> = sqlx::query_as(
            r#"
            SELECT id, network_id, referendum_id, index, block_hash, block_number, extrinsic_index, vote, balance, conviction, is_removed, subsquare_comment_cid, subsquare_comment_index, has_coi, is_forced
            FROM pdao_vote
            WHERE referendum_id = $1
            ORDER BY id DESC
            LIMIT 1
            "#,
        )
            .bind(referendum_id as i32)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(db_vote) = db_vote {
            Ok(Some(vote_record_into_vote(&db_vote)?))
        } else {
            Ok(None)
        }
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
        vote: Option<bool>,
        feedback: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO pdao_member_vote (vote_id, cid, network_id, referendum_id, index, address, vote, feedback)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT(vote_id, address) DO UPDATE
            SET vote = EXCLUDED.vote, feedback = EXCLUDED.feedback
            "#,
        )
            .bind(vote_id as i32)
            .bind(cid)
            .bind(network_id as i32)
            .bind(referendum_id as i32)
            .bind(referendum_index as i32)
            .bind(address)
            .bind(vote)
            .bind(feedback)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn get_member_votes(&self) -> anyhow::Result<Vec<MemberVote>> {
        let db_member_votes: Vec<MemberVoteRecord> = sqlx::query_as(
            r#"
            SELECT id, vote_id, cid, network_id, referendum_id, index, address, vote, feedback
            FROM pdao_member_vote
            ORDER BY id ASC
            "#,
        )
        .fetch_all(&self.connection_pool)
        .await?;
        let mut member_votes = Vec::new();
        for db_member_vote in db_member_votes.iter() {
            member_votes.push(member_vote_record_into_member_vote(db_member_vote)?);
        }
        Ok(member_votes)
    }

    pub async fn get_vote_member_votes(&self, vote_id: u32) -> anyhow::Result<Vec<MemberVote>> {
        let db_member_votes: Vec<MemberVoteRecord> = sqlx::query_as(
            r#"
            SELECT id, vote_id, cid, network_id, referendum_id, index, address, vote, feedback
            FROM pdao_member_vote
            WHERE vote_id = $1
            ORDER BY id ASC
            "#,
        )
        .bind(vote_id as i32)
        .fetch_all(&self.connection_pool)
        .await?;
        let mut member_votes = Vec::new();
        for db_member_vote in db_member_votes.iter() {
            member_votes.push(member_vote_record_into_member_vote(db_member_vote)?);
        }
        Ok(member_votes)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn save_pending_member_vote(
        &self,
        cid: &str,
        network_id: u32,
        referendum_id: u32,
        referendum_index: u32,
        address: &str,
        vote: Option<bool>,
        feedback: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO pdao_pending_member_vote (cid, network_id, referendum_id, index, address, vote, feedback)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT(referendum_id, address) DO UPDATE
            SET cid = EXCLUDED.cid, vote = EXCLUDED.vote, feedback = EXCLUDED.feedback
            RETURNING id
            "#,
        )
            .bind(cid)
            .bind(network_id as i32)
            .bind(referendum_id as i32)
            .bind(referendum_index as i32)
            .bind(address)
            .bind(vote)
            .bind(feedback)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn get_referendum_pending_member_votes(
        &self,
        referendum_id: u32,
    ) -> anyhow::Result<Vec<PendingMemberVote>> {
        let db_pending_member_votes: Vec<PendingMemberVoteRecord> = sqlx::query_as(
            r#"
            SELECT id, cid, network_id, referendum_id, index, address, vote, feedback
            FROM pdao_pending_member_vote
            WHERE referendum_id = $1
            ORDER BY id ASC
            "#,
        )
        .bind(referendum_id as i32)
        .fetch_all(&self.connection_pool)
        .await?;
        let mut pending_member_votes = Vec::new();
        for db_pending_member_vote in db_pending_member_votes.iter() {
            pending_member_votes.push(pending_member_vote_record_into_pending_member_vote(
                db_pending_member_vote,
            )?);
        }
        Ok(pending_member_votes)
    }

    pub async fn delete_pending_member_vote(&self, id: u32) -> anyhow::Result<bool> {
        let delete_result = sqlx::query("DELETE FROM pdao_pending_member_vote WHERE id = $1")
            .bind(id as i32)
            .execute(&self.connection_pool)
            .await?;
        Ok(delete_result.rows_affected() == 1)
    }

    pub async fn delete_referendum_pending_member_votes(
        &self,
        referendum_id: u32,
    ) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM pdao_pending_member_vote WHERE referendum_id = $1")
            .bind(referendum_id as i32)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }
}
