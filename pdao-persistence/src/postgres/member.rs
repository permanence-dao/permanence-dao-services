use crate::postgres::PostgreSQLStorage;
use pdao_types::substrate::account_id::AccountId;
use pdao_types::Member;
use sqlx::FromRow;
use std::str::FromStr;

#[derive(Debug, FromRow)]
struct MemberRow {
    pub id: i32,
    pub name: String,
    pub telegram_username: String,
    pub polkadot_address: String,
    pub polkadot_payment_address: String,
    pub kusama_address: String,
    pub kusama_payment_address: String,
    pub is_on_leave: bool,
}

impl PostgreSQLStorage {
    #[allow(clippy::type_complexity)]
    pub async fn get_all_members(&self) -> anyhow::Result<Vec<Member>> {
        let db_members: Vec<MemberRow> = sqlx::query_as::<_, MemberRow>(
            r#"
            SELECT id, name, telegram_username, polkadot_address, polkadot_payment_address, kusama_address, kusama_payment_address, is_on_leave
            FROM pdao_member
            ORDER BY id ASC
            "#,
        )
            .fetch_all(&self.connection_pool)
            .await?;
        let mut result = Vec::new();
        for db_member in db_members.iter() {
            result.push(Member {
                name: db_member.name.clone(),
                telegram_username: db_member.telegram_username.clone(),
                polkadot_address: AccountId::from_str(&db_member.polkadot_address)?,
                polkadot_payment_address: AccountId::from_str(&db_member.polkadot_payment_address)?,
                kusama_address: AccountId::from_str(&db_member.kusama_address)?,
                kusama_payment_address: AccountId::from_str(&db_member.kusama_payment_address)?,
                is_on_leave: db_member.is_on_leave,
            })
        }
        Ok(result)
    }
}
