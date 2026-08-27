// Copyright (c) 2026 DigiText. All rights reserved.
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
pub async fn connect(r_2ac: &str) -> sqlx::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(r_2ac)
        .await
}
pub async fn f_11d(r_2ad: &str) -> Result<PgPool, sqlx::Error> {
    connect(r_2ad).await
}
