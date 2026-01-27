// apps/backcli/src/main.rs

mod infisical_setup;
mod keycloak_setup;

use clap::{Arg, ArgAction, Command};
use secrecy::Secret;
use sqlx::MySqlPool;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

use user_api::keycloak::{KeycloakClient, KeycloakConfig};
use user_lib::repository::{RoleRepository, UserRepository, UserRoleRepository};
use user_lib::rootuser::{initialize_root_user, RootUserConfig};

// NOTE: sqlx::migrate!(...) paths are resolved relative to this crate's directory (apps/backcli)
static USER_LIB_MIGRATOR: sqlx::migrate::Migrator =
    sqlx::migrate!("../../libs/user-lib/migrations");

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let matches = Command::new("backcli")
        .about("Backender CLI utility")
        .arg(
            Arg::new("migrations")
                .long("migrations")
                .action(ArgAction::SetTrue)
                .help("Execute database migrations"),
        )
        .arg(
            Arg::new("delete")
                .long("delete")
                .action(ArgAction::SetTrue)
                .help("Revert migrations (down) for the selected library"),
        )
        .arg(
            Arg::new("user-lib")
                .long("user-lib")
                .action(ArgAction::SetTrue)
                .help("Target only user-lib migrations"),
        )
        .arg(
            Arg::new("init-root")
                .long("init-root")
                .action(ArgAction::SetTrue)
                .help("Initialize root user in Keycloak and database"),
        )
        .arg(
            Arg::new("setup-keycloak")
                .long("setup-keycloak")
                .action(ArgAction::SetTrue)
                .help("Setup Keycloak service account client and retrieve secret"),
        )
        .arg(
            Arg::new("setup-infisical")
                .long("setup-infisical")
                .action(ArgAction::SetTrue)
                .help("Setup Infisical secrets manager and retrieve machine identity credentials"),
        )
        .arg(
            Arg::new("store-secret")
                .long("store-secret")
                .action(ArgAction::SetTrue)
                .help("Store a secret in Infisical (requires --key and --value)"),
        )
        .arg(
            Arg::new("key")
                .long("key")
                .value_name("KEY")
                .help("Secret key name (used with --store-secret)"),
        )
        .arg(
            Arg::new("value")
                .long("value")
                .value_name("VALUE")
                .help("Secret value (used with --store-secret)"),
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

    if matches.get_flag("delete") {
        let result = if matches.get_flag("user-lib") {
            run_user_lib_delete().await
        } else {
            // In the future, support more libs here
            run_user_lib_delete().await
        };
        if let Err(e) = result {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }

    if matches.get_flag("init-root") {
        if let Err(e) = init_root_user().await {
            eprintln!("Error initializing root user: {e}");
            std::process::exit(1);
        }
    }

    if matches.get_flag("setup-keycloak") {
        if let Err(e) = setup_keycloak().await {
            eprintln!("Error setting up Keycloak: {e}");
            std::process::exit(1);
        }
    }

    if matches.get_flag("setup-infisical") {
        if let Err(e) = setup_infisical().await {
            eprintln!("Error setting up Infisical: {e}");
            std::process::exit(1);
        }
    }

    if matches.get_flag("store-secret") {
        let key = matches
            .get_one::<String>("key")
            .expect("--key is required with --store-secret");
        let value = matches
            .get_one::<String>("value")
            .expect("--value is required with --store-secret");

        if let Err(e) = store_secret(key, value).await {
            eprintln!("Error storing secret: {e}");
            std::process::exit(1);
        }
    }
}

async fn run_user_lib_migrations() -> Result<(), String> {
    let db_url =
        std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set".to_string())?;

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

async fn run_user_lib_delete() -> Result<(), String> {
    let db_url =
        std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set".to_string())?;

    let pool = MySqlPool::connect(&db_url)
        .await
        .map_err(|e| format!("Failed to connect to database: {e}"))?;

    println!("Reverting all migrations for user-lib...");
    // SQLx Migrator supports down migrations via `undo(target_version)`.
    // Passing `0` reverts all applied migrations because migration versions are positive timestamps.
    USER_LIB_MIGRATOR
        .undo(&pool, 0)
        .await
        .map_err(|e| format!("Revert failed: {e}"))?;

    println!("Data deleted successfully.");
    Ok(())
}

/// Initialize root user in Keycloak and database
async fn init_root_user() -> Result<(), String> {
    println!("Initializing root user...");

    // Load configuration from environment
    let mut config = RootUserConfig::from_env()?;
    let password = RootUserConfig::password_from_env()?;

    println!("Root user configuration:");
    println!("  Email: {}", config.email);
    println!("  Name: {} {}", config.first_name, config.last_name);

    // Step 1: Initialize Keycloak client
    let keycloak_config = KeycloakConfig::from_env();

    if !keycloak_config.is_configured() {
        return Err("Keycloak is not configured. Please set KEYCLOAK_CLIENT_SECRET".to_string());
    }

    let keycloak = Arc::new(KeycloakClient::new(keycloak_config));

    println!("Creating user in Keycloak...");

    // Step 2: Create user in Keycloak
    let keycloak_id = keycloak
        .create_user(
            &config.email,
            Some(&config.first_name),
            Some(&config.last_name),
            Some(&Secret::new(password)),
        )
        .await
        .map_err(|e| format!("Failed to create user in Keycloak: {e}"))?;

    println!("✓ User created in Keycloak with ID: {keycloak_id}");

    // Update config with keycloak_id
    config.keycloak_id = keycloak_id.clone();

    // Step 3: Connect to database
    let db_url =
        std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set".to_string())?;

    let pool = MySqlPool::connect(&db_url)
        .await
        .map_err(|e| format!("Failed to connect to database: {e}"))?;

    // Step 4: Initialize repositories
    let user_repo = UserRepository::new(pool.clone());
    let role_repo = RoleRepository::new(pool.clone());
    let user_role_repo = UserRoleRepository::new(pool.clone());

    println!("Creating user in database and assigning admin role...");

    // Step 5: Initialize root user in database
    let user = initialize_root_user(&user_repo, &role_repo, &user_role_repo, &config)
        .await
        .map_err(|e| {
            // If DB creation fails, we should delete from Keycloak
            let kc_clone = keycloak.clone();
            let kc_id = keycloak_id.clone();
            tokio::spawn(async move {
                if let Err(del_err) = kc_clone.delete_user(&kc_id).await {
                    eprintln!("WARNING: Failed to rollback Keycloak user: {del_err}");
                    eprintln!("Manual cleanup required for Keycloak user: {kc_id}");
                } else {
                    println!("✓ Rolled back Keycloak user due to database error");
                }
            });
            format!("Failed to initialize user in database: {e}")
        })?;

    println!("✓ User created in database with ID: {}", user.id);
    println!("✓ Admin role assigned to user");
    println!("\nRoot user initialized successfully!");
    println!("  User ID: {}", user.id);
    println!("  Keycloak ID: {}", user.keycloak_id);
    println!(
        "  Roles: {}",
        user.roles
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    Ok(())
}

/// Setup Keycloak service account client
async fn setup_keycloak() -> Result<(), String> {
    println!("Setting up Keycloak service account...\n");

    // Get the client ID from environment
    let client_id =
        std::env::var("KEYCLOAK_CLIENT_ID").unwrap_or_else(|_| "user-api-service".to_string());

    // Initialize Keycloak setup
    let setup = keycloak_setup::KeycloakSetup::from_env()?;

    // Create/retrieve the service account and get its secret
    let secret = setup.setup_service_account(&client_id).await?;

    println!("\n✓ Keycloak service account setup complete!");
    println!("\nAdd this to your .env.local file:");
    println!("KEYCLOAK_CLIENT_SECRET={secret}");

    Ok(())
}

/// Setup Infisical secrets manager
async fn setup_infisical() -> Result<(), String> {
    println!("Setting up Infisical secrets manager...\n");

    let setup = infisical_setup::InfisicalSetup::from_env()?;
    let credentials = setup.setup().await?;

    println!("\n✓ Infisical setup complete!");
    println!("\nAdd these to your .env.local file:");
    println!("INFISICAL_URL={}", credentials.url);
    println!("INFISICAL_CLIENT_ID={}", credentials.client_id);
    println!("INFISICAL_CLIENT_SECRET={}", credentials.client_secret);
    println!("INFISICAL_PROJECT_ID={}", credentials.project_id);
    println!("INFISICAL_ENVIRONMENT={}", credentials.environment);

    Ok(())
}

/// Store a secret in Infisical
async fn store_secret(key: &str, value: &str) -> Result<(), String> {
    let setup = infisical_setup::InfisicalSetup::from_env()?;
    setup.store_secret(key, value).await?;

    println!("✓ Secret '{key}' stored in Infisical");

    Ok(())
}
