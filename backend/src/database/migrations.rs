use sqlx::migrate::Migrator;
use std::path::Path;

pub async fn run_migrations() -> Result<(), sqlx::Error> {
    let migrator = Migrator::new(Path::new("./migrations")).await?;
    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL")?).await?;
    migrator.run(&pool).await?;
    Ok(())
}
