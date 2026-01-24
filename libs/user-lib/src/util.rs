use std::{str::FromStr, time::Duration};

use sqlx::{mysql::{MySqlConnectOptions, MySqlPoolOptions}, MySqlPool};
use tokio::time::sleep;

#[allow(dead_code)]
pub async fn connect_with_retry(database_url: &str, max_retries: u32) -> MySqlPool {
    let mut retries = 0;

    let connect_options = MySqlConnectOptions::from_str(database_url)
        .expect("Invalid DATABASE_URL")
        .to_owned();

    loop {
        match MySqlPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect_with(connect_options.clone())
            .await
        {
            Ok(pool) => return pool,
            Err(e) if retries < max_retries => {
                eprintln!("MySQL not ready yet (attempt {}): {:?}", retries + 1, e);
                retries += 1;
                sleep(Duration::from_secs(1)).await;
            }
            Err(e) => panic!("Failed to connect to MySQL after {} retries: {:?}", max_retries, e),
        }
    }
}