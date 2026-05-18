use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL not set");
    PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
