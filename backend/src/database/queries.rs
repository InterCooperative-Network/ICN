use sqlx::PgPool;
use crate::database::models::{Proposal, Vote, Contribution};

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

pub async fn query_shared_resources(pool: &PgPool, resource_type: &str, owner: Option<&str>) -> Result<Vec<Resource>, sqlx::Error> {
    let query = match owner {
        Some(owner) => {
            sqlx::query_as!(
                Resource,
                r#"
                SELECT * FROM resources
                WHERE resource_type = $1 AND owner = $2
                "#,
                resource_type,
                owner
            )
        }
        None => {
            sqlx::query_as!(
                Resource,
                r#"
                SELECT * FROM resources
                WHERE resource_type = $1
                "#,
                resource_type
            )
        }
    };

    let resources = query.fetch_all(pool).await?;
    Ok(resources)
}

pub async fn store_contribution(pool: &PgPool, contribution: &Contribution) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO contributions (did, score, timestamp)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        contribution.did,
        contribution.score,
        contribution.timestamp
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}

pub async fn retrieve_contributions(pool: &PgPool, did: &str) -> Result<Vec<Contribution>, sqlx::Error> {
    let contributions = sqlx::query_as!(
        Contribution,
        r#"
        SELECT id, did, score, timestamp FROM contributions
        WHERE did = $1
        "#,
        did
    )
    .fetch_all(pool)
    .await?;

    Ok(contributions)
}
