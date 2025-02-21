use crate::postgres::PostgreSQLStorage;
use pdao_types::substrate::account_id::AccountId;
use pdao_types::Member;
use std::str::FromStr;

impl PostgreSQLStorage {
    pub async fn get_all_members(&self) -> anyhow::Result<Vec<Member>> {
        let db_members: Vec<(i32, String, String, String, String, String, String)> = sqlx::query_as(
            r#"
            SELECT id, name, telegram_username, polkadot_address, polkadot_payment_address, kusama_address, kusama_payment_address
            FROM pdao_member
            ORDER BY id ASC
            "#,
        )
            .fetch_all(&self.connection_pool)
            .await?;
        let mut result = Vec::new();
        for db_member in db_members.iter() {
            result.push(Member {
                name: db_member.1.clone(),
                telegram_username: db_member.2.clone(),
                polkadot_address: AccountId::from_str(&db_member.3)?,
                polkadot_payment_address: AccountId::from_str(&db_member.4)?,
                kusama_address: AccountId::from_str(&db_member.5)?,
                kusama_payment_address: AccountId::from_str(&db_member.6)?,
            })
        }
        Ok(result)
    }
}
