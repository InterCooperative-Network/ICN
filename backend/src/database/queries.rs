use sqlx::PgPool;
use crate::database::models::{Proposal, Vote};

pub async fn create_proposal(pool: &PgPool, proposal: &Proposal) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO proposals (title, description, created_by, ends_at, created_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        proposal.title,
        proposal.description,
        proposal.created_by,
        proposal.ends_at,
        proposal.created_at
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}

pub async fn record_vote(pool: &PgPool, vote: &Vote) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO votes (proposal_id, voter, approve)
        VALUES ($1, $2, $3)
        "#,
        vote.proposal_id,
        vote.voter,
        vote.approve
    )
    .execute(pool)
    .await?;

    Ok(())
}
