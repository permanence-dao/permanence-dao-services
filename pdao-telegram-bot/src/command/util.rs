use crate::CONFIG;
use pdao_opensquare_client::OpenSquareClient;
use pdao_persistence::postgres::PostgreSQLStorage;
use pdao_subsquare_client::SubSquareClient;
use pdao_types::governance::opensquare::{
    OpenSquareReferendum, OpenSquareReferendumVote, OpenSquareVote,
};
use pdao_types::governance::policy::VotingPolicy;
use pdao_types::governance::subsquare::SubSquareReferendum;
use pdao_types::governance::track::Track;
use pdao_types::governance::{Referendum, ReferendumStatus};
use pdao_types::substrate::account_id::AccountId;
use pdao_types::substrate::chain::Chain;
use pdao_types::Member;

pub(super) fn require_thread(thread_id: Option<i32>) -> anyhow::Result<i32> {
    if let Some(thread_id) = thread_id {
        Ok(thread_id)
    } else {
        Err(anyhow::Error::msg(
            "This command can only be called from a referendum topic.",
        ))
    }
}

pub(super) async fn require_subsquare_referendum(
    subsquare_client: &SubSquareClient,
    chain: &Chain,
    referendum_index: u32,
) -> anyhow::Result<SubSquareReferendum> {
    if let Some(referendum) = subsquare_client
        .fetch_referendum(chain, referendum_index)
        .await?
    {
        Ok(referendum)
    } else {
        Err(anyhow::Error::msg(
            "Referendum not found on SubSquare. Contact admin.",
        ))
    }
}

pub(super) fn require_opensquare_cid(db_referendum: &Referendum) -> anyhow::Result<&str> {
    if let Some(cid) = db_referendum.opensquare_cid.as_deref() {
        Ok(cid)
    } else {
        Err(anyhow::Error::msg(
            "OpenSquare CID not found in the database referendum record. Contact admin.",
        ))
    }
}

pub(super) async fn require_opensquare_referendum(
    opensquare_client: &OpenSquareClient,
    cid: &str,
) -> anyhow::Result<OpenSquareReferendum> {
    if let Some(referendum) = opensquare_client.fetch_referendum(cid).await? {
        Ok(referendum)
    } else {
        Err(anyhow::Error::msg(
            "Referendum not found on OpenSquare. Contact admin.",
        ))
    }
}

pub(super) async fn require_db_referendum(
    postgres: &PostgreSQLStorage,
    chat_id: i64,
    thread_id: i32,
) -> anyhow::Result<Referendum> {
    if let Some(referendum) = postgres
        .get_referendum_by_telegram_chat_and_thread_id(chat_id, thread_id)
        .await?
    {
        Ok(referendum)
    } else {
        Err(anyhow::Error::msg(
            "Referendum not found in the storage. Contact admin.",
        ))
    }
}

pub(super) fn require_db_referendum_is_active(db_referendum: &Referendum) -> anyhow::Result<()> {
    if db_referendum.is_terminated {
        Err(anyhow::Error::msg(
            "Referendum has been terminated. Cannot remove vote.",
        ))
    } else {
        Ok(())
    }
}

pub(super) async fn require_opensquare_votes(
    opensquare_client: &OpenSquareClient,
    opensquare_cid: &str,
    member_account_ids: &[AccountId],
) -> anyhow::Result<Vec<OpenSquareReferendumVote>> {
    if let Some(opensquare_votes) = opensquare_client
        .fetch_referendum_votes(opensquare_cid)
        .await?
    {
        Ok(opensquare_votes
            .iter()
            .filter(|v| member_account_ids.contains(&v.voter))
            .cloned()
            .collect())
    } else {
        Err(anyhow::Error::msg(
            "Referendum not found on OpenSquare by CID. Contact admin.",
        ))
    }
}

pub(super) fn require_opensquare_referendum_active(
    opensquare_referendum: &OpenSquareReferendum,
) -> anyhow::Result<()> {
    if opensquare_referendum.status != "active" {
        Err(anyhow::Error::msg("OpenSquare referendum is not active."))
    } else {
        Ok(())
    }
}

pub(super) fn require_subsquare_referendum_active(
    subsquare_referendum: &SubSquareReferendum,
) -> anyhow::Result<()> {
    if !(subsquare_referendum.state.status == ReferendumStatus::Deciding
        || subsquare_referendum.state.status == ReferendumStatus::Preparing
        || subsquare_referendum.state.status == ReferendumStatus::Queueing
        || subsquare_referendum.state.status == ReferendumStatus::Confirming)
    {
        Err(anyhow::Error::msg(
            "Cannot complete action for referendum status: {}",
        ))
    } else {
        Ok(())
    }
}

pub(super) fn get_vote_counts(votes: &[OpenSquareReferendumVote]) -> (u32, u32, u32) {
    let mut aye_count = 0;
    let mut nay_count = 0;
    let mut abstain_count = 0;
    for vote in votes.iter() {
        if vote.choices.contains(&OpenSquareVote::Aye) {
            aye_count += 1;
        } else if vote.choices.contains(&OpenSquareVote::Nay) {
            nay_count += 1;
        } else if vote.choices.contains(&OpenSquareVote::Abstain) {
            abstain_count += 1;
        }
    }
    (aye_count, nay_count, abstain_count)
}

pub(super) fn require_voting_policy(track: &Track) -> anyhow::Result<VotingPolicy> {
    if let Some(voting_policy) = VotingPolicy::voting_policy_for_track(track) {
        Ok(voting_policy)
    } else {
        Err(anyhow::Error::msg(format!(
            "No voting policy is defined for {}.",
            track.name(),
        )))
    }
}

pub(super) fn require_voting_admin(username: &str) -> anyhow::Result<()> {
    if !CONFIG.voter.voting_admin_usernames.contains(username) {
        Err(anyhow::Error::msg(
            "This command can only be called by a voting admin.",
        ))
    } else {
        Ok(())
    }
}

pub(super) async fn require_member(
    postgres: &PostgreSQLStorage,
    username: &str,
) -> anyhow::Result<Member> {
    if let Some(member) = postgres.get_member_by_username(username).await? {
        Ok(member)
    } else {
        Err(anyhow::Error::msg(format!(
            "@{username} is not registered as a member."
        )))
    }
}

pub fn round_half_down(x: f64) -> f64 {
    (x - 0.5).ceil()
}
