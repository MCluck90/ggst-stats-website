mod website;
use dotenv::dotenv;
use tokio;
mod caching;
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
    website::start().await;
}

