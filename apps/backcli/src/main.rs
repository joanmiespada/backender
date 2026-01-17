// apps/backcli/src/main.rs

use clap::{Arg, ArgAction, Command};
use sqlx::MySqlPool;

// NOTE: sqlx::migrate!(...) paths are resolved relative to this crate's directory (apps/backcli)
static USER_LIB_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../libs/user-lib/migrations");

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
        let result = if matches.get_flag("user-lib") {
            run_user_lib_migrations().await
        } else {
            // In the future, support more libs here
            run_user_lib_migrations().await
        };

        if let Err(e) = result {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

async fn run_user_lib_migrations() -> Result<(), String> {
    let db_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set".to_string())?;

    let pool = MySqlPool::connect(&db_url)
        .await
        .map_err(|e| format!("Failed to connect to database: {e}"))?;

    println!("Running migrations for user-lib...");
    USER_LIB_MIGRATOR
        .run(&pool)
        .await
        .map_err(|e| format!("Migration failed: {e}"))?;

    println!("Migrations applied successfully.");
    Ok(())
}