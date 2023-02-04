use sqlx::{postgres::PgPoolOptions, pool::PoolConnection, Postgres};

pub async fn start(mut pool: PoolConnection<Postgres>) {
    let row = sqlx::query!("SELECT COUNT(*) FROM matches")
            .fetch_one(&mut pool)
            .await
            .unwrap();
    println!("{}", row.count.unwrap());
}