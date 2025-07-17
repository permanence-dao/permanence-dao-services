use crate::postgres::PostgreSQLStorage;
use pdao_types::substrate::account_id::AccountId;
use pdao_types::substrate::chain::Chain;
use pdao_types::{Member, MembershipType};
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
    pub membership_type_code: String,
}

impl TryInto<Member> for MemberRow {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Member, Self::Error> {
        Ok(Member {
            id: self.id as u32,
            name: self.name.clone(),
            telegram_username: self.telegram_username.clone(),
            polkadot_address: AccountId::from_str(&self.polkadot_address)?,
            polkadot_payment_address: AccountId::from_str(&self.polkadot_payment_address)?,
            kusama_address: AccountId::from_str(&self.kusama_address)?,
            kusama_payment_address: AccountId::from_str(&self.kusama_payment_address)?,
            is_on_leave: self.is_on_leave,
            membership_type: MembershipType::from(self.membership_type_code.as_str()),
        })
    }
}

impl PostgreSQLStorage {
    pub async fn mark_member_leave(&self, member_id: u32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO pdao_member_leave (member_id)
            VALUES ($1)
            RETURNING id
            "#,
        )
        .bind(member_id as i32)
        .execute(&self.connection_pool)
        .await?;
        sqlx::query(
            r#"
            UPDATE pdao_member SET is_on_leave = TRUE
            WHERE id = $1
            RETURNING id
            "#,
        )
        .bind(member_id as i32)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn mark_member_return(&self, member_id: u32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO pdao_member_return (member_id)
            VALUES ($1)
            RETURNING id
            "#,
        )
        .bind(member_id as i32)
        .execute(&self.connection_pool)
        .await?;
        sqlx::query(
            r#"
            UPDATE pdao_member SET is_on_leave = FALSE
            WHERE id = $1
            RETURNING id
            "#,
        )
        .bind(member_id as i32)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn get_member_by_username(&self, username: &str) -> anyhow::Result<Option<Member>> {
        let maybe_db_member: Option<MemberRow> = sqlx::query_as::<_, MemberRow>(
            r#"
            SELECT id, name, telegram_username, polkadot_address, polkadot_payment_address, kusama_address, kusama_payment_address, is_on_leave, membership_type_code
            FROM pdao_member
            WHERE telegram_username = $1
            "#
        )
            .bind(username)
            .fetch_optional(&self.connection_pool)
            .await?;

        if let Some(db_member) = maybe_db_member {
            Ok(Some(db_member.try_into()?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_members(&self, include_on_leave: bool) -> anyhow::Result<Vec<Member>> {
        let on_leave_filter = if include_on_leave {
            ""
        } else {
            "WHERE is_on_leave = FALSE"
        };
        let db_members: Vec<MemberRow> = sqlx::query_as::<_, MemberRow>(
            format!(r#"
            SELECT id, name, telegram_username, polkadot_address, polkadot_payment_address, kusama_address, kusama_payment_address, is_on_leave, membership_type_code
            FROM pdao_member {on_leave_filter}
            ORDER BY id ASC
            "#).as_str(),
        )
            .fetch_all(&self.connection_pool)
            .await?;
        let mut result = Vec::new();
        for db_member in db_members.iter() {
            result.push(Member {
                id: db_member.id as u32,
                name: db_member.name.clone(),
                telegram_username: db_member.telegram_username.clone(),
                polkadot_address: AccountId::from_str(&db_member.polkadot_address)?,
                polkadot_payment_address: AccountId::from_str(&db_member.polkadot_payment_address)?,
                kusama_address: AccountId::from_str(&db_member.kusama_address)?,
                kusama_payment_address: AccountId::from_str(&db_member.kusama_payment_address)?,
                is_on_leave: db_member.is_on_leave,
                membership_type: MembershipType::from(db_member.membership_type_code.as_str()),
            })
        }
        Ok(result)
    }

    pub async fn get_all_member_account_ids_for_chain(
        &self,
        include_on_leave: bool,
        network_id: u32,
    ) -> anyhow::Result<Vec<AccountId>> {
        let members = self.get_all_members(include_on_leave).await?;
        let chain = Chain::from_id(network_id);
        let member_account_ids = match chain.chain.as_str() {
            "polkadot" => members.iter().map(|m| m.polkadot_address).collect(),
            "kusama" => members.iter().map(|m| m.kusama_address).collect(),
            _ => Vec::new(),
        };
        Ok(member_account_ids)
    }
}
