mod caching;
mod characters;
mod ggst_api;
mod requests;
mod responses;
mod website;
// use characters::{convert_to_character, Character};
use chrono::NaiveDateTime;
use dotenv::dotenv;
use ggst_api::*;
use responses::Replay;
use sqlx::{pool::PoolConnection, postgres::PgPoolOptions, Postgres};
use std::env;
use tokio;
use tokio::task;

const MATCHES_PER_PAGE: f32 = 127.0;

#[tokio::main]
async fn main() {
    match dotenv() {
        Ok(_) => println!("Sucessfully loaded environment variables."),
        Err(err) => {
            println!("Error parsing environment variables:\n{}", err);
            panic!();
        }
    }
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let mut task_set = task::JoinSet::new();
    task_set.spawn(fetch_and_add_matches(pool.acquire().await.unwrap()));
    task_set.spawn(website::start());

    loop {
        let _ = task_set.join_next().await;
        if task_set.len() == 0 {
            break;
        }
    }
}

async fn fetch_and_add_matches(mut pool: PoolConnection<Postgres>) {
    let mut pages: u64 = 5;
    loop {
        let replays: Vec<Replay> = match get_replays(pages).await {
            Ok(r) => r,
            Err(err) => {
                println!("Error: {}", err);
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                continue;
            } // I should log these errors somewhere.
        };

        let old_count_row = sqlx::query!("SELECT COUNT(*) FROM matches")
            .fetch_one(&mut pool)
            .await
            .unwrap();
        let old_count = old_count_row.count.unwrap();
        for replay in &replays {
            let time =
                NaiveDateTime::parse_from_str(&replay.timestamp, "%Y-%m-%d %H:%M:%S").unwrap();
            sqlx::query!(
                r#"INSERT into MATCHES (player1, player2, player1_char, player2_char, winner, timestamp, floor, player1_id, player2_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT DO NOTHING"#,
                replay.player1.name,
                replay.player2.name,
                replay.player1_character as i32,
                replay.player2_character as i32,
                replay.winner as i32,
                time,
                replay.floor as i32,
                replay.player1.id,
                replay.player2.id
            ).execute(&mut pool).await.unwrap();
        }

        let new_count_row = sqlx::query!("SELECT COUNT(*) FROM matches")
            .fetch_one(&mut pool)
            .await
            .unwrap();
        let new_count = new_count_row.count.unwrap();
        let total_added: i64 = new_count - old_count;
        let length: i64 = replays.len().try_into().unwrap();

        println!(
            "New matches added: {}\nWe had received: {}\nCurrent count: {}",
            total_added, length, new_count
        );
        let calculated = (total_added as f32 / MATCHES_PER_PAGE).ceil();
        pages = calculated as u64 + 2;

        println!("New page count: {}\n", pages);
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
