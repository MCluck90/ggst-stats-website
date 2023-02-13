use crate::caching;
use askama::Template;
use axum::{
    http::StatusCode, response::IntoResponse, routing::get, routing::get_service, Extension, Router,
};
use rustis::client::Client;
use serde::Serialize;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::Pool;
use sqlx::Postgres;
use std::env;
use tower_http::services::ServeDir;
pub async fn start() {
    let redis_client = Client::connect("redis://127.0.0.1/").await.unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let app = Router::new()
        .nest_service(
            "/static",
            get_service(ServeDir::new("static")).handle_error(handle_error),
        )
        .nest_service(
            "/img",
            get_service(ServeDir::new("img")).handle_error(handle_error),
        )
        .route("/", get(root))
        .layer(Extension(pool))
        .layer(Extension(redis_client));

    // `axum::Server` is a re-export of `hyper::Server`
    let ip = env::var("IP").unwrap();
    let port = env::var("PORT").unwrap();
    let addr = format!("{}:{}", ip, port);
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root(pool: Extension<Pool<Postgres>>, redis: Extension<Client>) -> Index {
    let mut sql_conn = pool.acquire().await.unwrap();
    let mut redis_client = redis.0;
    let is_cached = caching::is_cached("index", &mut redis_client);

    match is_cached.await {
        Some(cache) => {
            let index: Index = serde_json::from_str(&cache).unwrap();
            index
        }
        None => {
            let rows = sqlx::query!(
                "(SELECT COUNT(*) FROM matches) UNION ALL
                    (SELECT COUNT(*) FROM matches WHERE floor = 99) UNION ALL
                    (SELECT COUNT(*) FROM matches WHERE floor between 8 and 10) UNION ALL
                    (SELECT COUNT(*) FROM matches WHERE floor < 8)"
            )
            .fetch_all(&mut sql_conn)
            .await
            .unwrap();

            let mut nums: Vec<i64> = Vec::new();
            for row in rows {
                let num = row.count.unwrap();
                nums.push(num);
            }
            let index = Index::new(nums);
            caching::store_cache("index", &mut redis_client, index).await;
            index
        }
    }
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

#[derive(Template, Serialize, Deserialize, Copy, Clone)]
#[template(path = "index.html")]
pub struct Index {
    count: i64,
    celestial: i64,
    eight_to_ten: i64,
    seven_and_below: i64,
}

impl Index {
    fn new(nums: Vec<i64>) -> Index {
        Index {
            count: nums[0],
            celestial: nums[1],
            eight_to_ten: nums[2],
            seven_and_below: nums[3],
        }
    }
}
