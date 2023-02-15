use crate::caching;
use askama::Template;
use axum::{
    extract::Path, http::StatusCode, response::IntoResponse, routing::get, routing::get_service,
    Extension, Router,
};
use rustis::client::Client;
use serde::Deserialize;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::Pool;
use sqlx::Postgres;
use stats_shared;
use stats_shared::character;
use stats_shared::character::Character;
use std::collections::HashMap;
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
        .nest_service(
            "/sql",
            get_service(ServeDir::new("sql")).handle_error(handle_error),
        )
        .route("/", get(index))
        .route("/chara/:chara", get(character))
        .layer(Extension(pool))
        .layer(Extension(redis_client));

    let ip = env::var("IP").unwrap();
    let port = env::var("PORT").unwrap();
    let addr = format!("{}:{}", ip, port);
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index(pool: Extension<Pool<Postgres>>, redis: Extension<Client>) -> Index {
    let mut sql_conn = pool.acquire().await.unwrap();
    let mut redis_client = redis.0;
    let is_cached = caching::is_cached("index".to_string(), &mut redis_client);

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
            let chariter = stats_shared::character::get_iter();
            let (upper_char, lower_char): (Vec<Character>, Vec<Character>) =
                chariter.partition(|n| *n as u8 % 2 == 0);
            let upper = upper_char.iter().map(|a| a.get_shorthand()).collect();
            let lower = lower_char.iter().map(|a| a.get_shorthand()).collect();
            let index = Index::new(nums, upper, lower);
            caching::store_cache("index".to_string(), &mut redis_client, &index).await;
            index
        }
    }
}

async fn character(
    pool: Extension<Pool<Postgres>>,
    redis: Extension<Client>,
    Path(chara): Path<String>,
) -> CharacterPage {
    let chara_enum = character::chara_from_shorthand(chara.clone()).unwrap();
    let mut sql_conn = pool.acquire().await.unwrap();
    let mut redis_client = redis.0;
    let key = format!("chara:{}", chara);
    let is_cached = caching::is_cached(key.clone(), &mut redis_client);

    match is_cached.await {
        Some(cache) => {
            let page: CharacterPage = serde_json::from_str(&cache).unwrap();
            return page;
        }
        None => {
            let charnum = chara_enum as i64;
            let rows = sqlx::query!(
                "SELECT floor FROM MATCHES WHERE (player1_char = $1 OR player2_char = $1)",
                charnum.to_string()
            )
            .fetch_all(&mut sql_conn)
            .await
            .unwrap();

            let mut matches: HashMap<String, i64> = HashMap::new();
            for x in 1..11 {
                matches.insert(x.to_string(), 0);
            }
            matches.insert("99".to_string(), 0);

            for row in rows {
                let floor = row.floor.to_string();
                matches.insert(floor.clone(), matches[&floor] + 1);
            }

            let mut hash: HashMap<String, i64> = HashMap::new();
            hash.insert("a".to_string(), 2);
            let page = CharacterPage::new(chara_enum, matches, hash);
            caching::store_cache(key, &mut redis_client, &page).await;
            return page;
        }
    }
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

#[derive(Template, Serialize, Deserialize, Clone)]
#[template(path = "index.html")]
pub struct Index {
    count: i64,
    celestial: i64,
    eight_to_ten: i64,
    seven_and_below: i64,
    upper_column: Vec<String>,
    lower_column: Vec<String>,
}

impl Index {
    fn new(nums: Vec<i64>, upper: Vec<String>, lower: Vec<String>) -> Index {
        Index {
            count: nums[0],
            celestial: nums[1],
            eight_to_ten: nums[2],
            seven_and_below: nums[3],
            upper_column: upper,
            lower_column: lower,
        }
    }
}

#[derive(Template, Serialize, Deserialize, Clone)]
#[template(path = "chara.html")]
pub struct CharacterPage {
    shorthand: String,
    full_name: String,
    floor_games: HashMap<String, i64>,
    matchups: HashMap<String, i64>,
}

impl CharacterPage {
    fn new(
        character: Character,
        floor_games: HashMap<String, i64>,
        win_ratio: HashMap<String, i64>,
    ) -> CharacterPage {
        CharacterPage {
            shorthand: character.get_shorthand(),
            full_name: character.get_full_name(),
            floor_games: floor_games,
            matchups: win_ratio,
        }
    }
}
