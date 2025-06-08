// apps/backcli/src/main.rs

use clap::{Arg, ArgAction, Command};
use sqlx::{migrate::Migrator, MySqlPool};
use std::path::Path;
use std::process;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let matches = Command::new("backcli")
        .about("Backender CLI utility")
        .arg(
            Arg::new("migrations")
                .long("migrations")
                .action(ArgAction::SetTrue)
                .help("Execute database migrations"),
        )
        .arg(
            Arg::new("user-lib")
                .long("user-lib")
                .action(ArgAction::SetTrue)
                .help("Target only user-lib migrations"),
        )
        .get_matches();

    if matches.get_flag("migrations") {
        if matches.get_flag("user-lib") {
            run_user_lib_migrations().await;
        } else {
            // In the future, support more libs here
            run_user_lib_migrations().await;
        }
    }
}

async fn run_user_lib_migrations() {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPool::connect(&db_url)
        .await
        .expect("Failed to connect to database");

    let migrator_path = Path::new("./libs/user-lib/migrations");
    let migrator = Arc::new(Migrator::new(migrator_path).await.expect("Invalid migrator"));

    println!("Running migrations for user-lib...");
    if let Err(e) = migrator.run(&pool).await {
        eprintln!("Migration failed: {}", e);
        process::exit(1);
    }
    println!("Migrations applied successfully.");
}