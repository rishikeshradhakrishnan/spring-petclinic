use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub async fn init_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:petclinic.db?mode=rwc")
        .await
        .expect("Failed to create database pool");

    // Run schema
    let schema = include_str!("../db/schema.sql");
    for statement in schema.split(';') {
        let stmt = statement.trim();
        if !stmt.is_empty() {
            sqlx::query(stmt)
                .execute(&pool)
                .await
                .unwrap_or_else(|e| {
                    log::warn!("Schema statement skipped: {e}");
                    Default::default()
                });
        }
    }

    // Check if data already exists
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vets")
        .fetch_one(&pool)
        .await
        .unwrap_or((0,));

    if count.0 == 0 {
        let data = include_str!("../db/data.sql");
        for statement in data.split(';') {
            let stmt = statement.trim();
            if !stmt.is_empty() {
                sqlx::query(stmt)
                    .execute(&pool)
                    .await
                    .unwrap_or_else(|e| {
                        log::warn!("Data statement skipped: {e}");
                        Default::default()
                    });
            }
        }
        log::info!("Seed data loaded successfully");
    }

    pool
}
