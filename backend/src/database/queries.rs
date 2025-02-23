use sqlx::PgPool;
use crate::database::models::{Proposal, Vote, Contribution, Federation, Resource};

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
        INSERT INTO contributions (did, score, timestamp, zk_snark_proof)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        contribution.did,
        contribution.score,
        contribution.timestamp,
        contribution.zk_snark_proof
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}

pub async fn retrieve_contributions(pool: &PgPool, did: &str) -> Result<Vec<Contribution>, sqlx::Error> {
    let contributions = sqlx::query_as!(
        Contribution,
        r#"
        SELECT id, did, score, timestamp, zk_snark_proof FROM contributions
        WHERE did = $1
        "#,
        did
    )
    .fetch_all(pool)
    .await?;

    Ok(contributions)
}

pub async fn store_proposal(pool: &PgPool, proposal: &Proposal) -> Result<i64, sqlx::Error> {
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

pub async fn store_vote(pool: &PgPool, vote: &Vote) -> Result<(), sqlx::Error> {
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

pub async fn create_federation(pool: &PgPool, federation: &Federation) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO federations (name, description, created_at)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        federation.name,
        federation.description,
        federation.created_at
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}

pub async fn update_federation_status(pool: &PgPool, federation_id: i64, status: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE federations
        SET status = $1
        WHERE id = $2
        "#,
        status,
        federation_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn dissolve_federation(pool: &PgPool, federation_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM federations
        WHERE id = $1
        "#,
        federation_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_federation_status(pool: &PgPool, federation_id: i64) -> Result<String, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT status FROM federations
        WHERE id = $1
        "#,
        federation_id
    )
    .fetch_one(pool)
    .await?;

    Ok(row.status)
}

pub async fn get_federation_assets(pool: &PgPool, federation_id: i64) -> Result<Vec<Resource>, sqlx::Error> {
    let resources = sqlx::query_as!(
        Resource,
        r#"
        SELECT * FROM resources
        WHERE federation_id = $1
        "#,
        federation_id
    )
    .fetch_all(pool)
    .await?;

    Ok(resources)
}

pub async fn get_federation_debts(pool: &PgPool, federation_id: i64) -> Result<Vec<Resource>, sqlx::Error> {
    let resources = sqlx::query_as!(
        Resource,
        r#"
        SELECT * FROM resources
        WHERE federation_id = $1 AND type = 'debt'
        "#,
        federation_id
    )
    .fetch_all(pool)
    .await?;

    Ok(resources)
}

pub async fn apply_reputation_decay(pool: &PgPool, did: &str, decay_rate: f64) -> Result<(), sqlx::Error> {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as f64;
    let contributions = sqlx::query_as!(
        Contribution,
        r#"
        SELECT score, timestamp FROM contributions WHERE did = $1
        "#,
        did
    )
    .fetch_all(pool)
    .await?;

    for contribution in contributions {
        let age = now - contribution.timestamp;
        let decayed_score = (contribution.score as f64 * (-decay_rate * age).exp()) as i64;
        sqlx::query!(
            r#"
            UPDATE contributions SET score = $1 WHERE did = $2 AND timestamp = $3
            "#,
            decayed_score,
            did,
            contribution.timestamp
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn handle_sybil_resistance(pool: &PgPool, did: &str, reputation_score: i64) -> Result<(), sqlx::Error> {
    // Placeholder logic for handling Sybil resistance
    Ok(())
}
