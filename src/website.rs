use askama::Template;
use axum::{
    extract::Path,
    handler::Handler,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    routing::get_service,
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::chrono::NaiveDate;
use sqlx::PgPool;
use sqlx::Pool;
use sqlx::Postgres;
use std::env;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::ServeDir;

pub async fn start() {
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
        .layer(Extension(pool));

    // `axum::Server` is a re-export of `hyper::Server`
    let ip = env::var("IP").unwrap();
    let port = env::var("PORT").unwrap();
    let addr = format!("{}:{}", ip, port);
    println!("{}", addr);
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root(pool: Extension<Pool<Postgres>>) -> Index {
    let total = match sqlx::query!("SELECT COUNT(*) FROM matches")
        .fetch_one(&pool.0)
        .await
    {
        Ok(a) => match a.count {
            Some(b) => b,
            None => 0,
        },
        Err(_) => 0,
    };
    let celestial = match sqlx::query!("SELECT COUNT(*) FROM matches WHERE floor = 99")
        .fetch_one(&pool.0)
        .await
    {
        Ok(a) => match a.count {
            Some(b) => b,
            None => 0,
        },
        Err(_) => 0,
    };
    let eight_to_ten =
        match sqlx::query!("SELECT COUNT(*) FROM matches WHERE floor between 8 and 10")
            .fetch_one(&pool.0)
            .await
        {
            Ok(a) => match a.count {
                Some(b) => b,
                None => 0,
            },
            Err(_) => 0,
        };
    let seven_and_below = match sqlx::query!("SELECT COUNT(*) FROM matches WHERE floor < 8")
        .fetch_one(&pool.0)
        .await
    {
        Ok(a) => match a.count {
            Some(b) => b,
            None => 0,
        },
        Err(_) => 0,
    };
    Index {
        count: total,
        celestial: celestial,
        eight_to_ten: eight_to_ten,
        seven_and_below: seven_and_below,
    }
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index {
    count: i64,
    celestial: i64,
    eight_to_ten: i64,
    seven_and_below: i64,
}
