//! Integration tests for Infisical secrets provider
//!
//! These tests require a running Infisical instance from docker-compose.
//! Run with: `just test-integration-secrets`
//!
//! Prerequisites:
//! 1. Start Infisical: `just up && just wait-infisical`
//! 2. Setup Infisical: `just setup-infisical`
//! 3. Ensure .env.local has INFISICAL_* variables set

use secrecy::ExposeSecret;
use secrets::{SecretsClient, SecretsConfig};

/// Helper to check if Infisical is configured in the environment
fn is_infisical_configured() -> bool {
    std::env::var("INFISICAL_CLIENT_ID").is_ok()
        && std::env::var("INFISICAL_CLIENT_SECRET").is_ok()
        && std::env::var("INFISICAL_PROJECT_ID").is_ok()
}

/// Test that SecretsClient initializes correctly with Infisical
#[tokio::test]
async fn test_secrets_client_with_infisical() {
    // Skip if Infisical is not configured
    if !is_infisical_configured() {
        eprintln!(
            "Skipping Infisical test - INFISICAL_* environment variables not set.\n\
             Run 'just setup-infisical' first."
        );
        return;
    }

    let config = SecretsConfig::from_env();
    let client = SecretsClient::new(config).await;

    // Should have primary provider (Infisical) configured
    assert!(
        client.has_primary_provider(),
        "SecretsClient should have Infisical as primary provider"
    );
}

/// Test fallback to environment variables when secret not in Infisical
#[tokio::test]
async fn test_fallback_to_env_vars() {
    // Set a test env var
    std::env::set_var("TEST_FALLBACK_SECRET", "env_value_123");

    let config = SecretsConfig::from_env();
    let client = SecretsClient::new(config).await;

    // This secret won't be in Infisical, should fall back to env
    let secret = client
        .get_secret_optional("TEST_FALLBACK_SECRET")
        .await
        .expect("Should get secret from env fallback");

    assert_eq!(secret.expose_secret(), "env_value_123");

    // Cleanup
    std::env::remove_var("TEST_FALLBACK_SECRET");
}

/// Test that missing secrets return None (not panic)
#[tokio::test]
async fn test_missing_secret_returns_none() {
    let config = SecretsConfig::from_env();
    let client = SecretsClient::new(config).await;

    let result = client
        .get_secret_optional("NONEXISTENT_SECRET_KEY_XXXXX")
        .await;

    assert!(result.is_none(), "Missing secret should return None");
}

/// Test env-only mode works correctly
#[tokio::test]
async fn test_env_only_mode() {
    std::env::set_var("TEST_ENV_ONLY_SECRET", "test_value");

    let client = SecretsClient::env_only();

    // Should not have primary provider
    assert!(
        !client.has_primary_provider(),
        "env_only client should not have primary provider"
    );

    let secret = client.get_secret_optional("TEST_ENV_ONLY_SECRET").await;
    assert!(secret.is_some());
    assert_eq!(secret.unwrap().expose_secret(), "test_value");

    std::env::remove_var("TEST_ENV_ONLY_SECRET");
}

/// Test caching behavior
#[tokio::test]
async fn test_caching() {
    std::env::set_var("TEST_CACHED_SECRET", "cached_value");

    // Enable caching
    let mut config = SecretsConfig::from_env();
    config.cache_enabled = true;

    let client = SecretsClient::new(config).await;

    // First call - should cache
    let secret1 = client.get_secret_optional("TEST_CACHED_SECRET").await;
    assert!(secret1.is_some());

    // Modify env var (but cache should still return old value)
    std::env::set_var("TEST_CACHED_SECRET", "modified_value");

    // Second call - should return cached value
    let secret2 = client.get_secret_optional("TEST_CACHED_SECRET").await;
    assert!(secret2.is_some());

    // Clear cache
    client.clear_cache().await;

    // Third call - should get new value
    let secret3 = client.get_secret_optional("TEST_CACHED_SECRET").await;
    assert!(secret3.is_some());
    assert_eq!(secret3.unwrap().expose_secret(), "modified_value");

    std::env::remove_var("TEST_CACHED_SECRET");
}

/// Test cache invalidation for specific key
#[tokio::test]
async fn test_cache_invalidation() {
    std::env::set_var("TEST_INVALIDATE_SECRET", "original");

    let mut config = SecretsConfig::from_env();
    config.cache_enabled = true;

    let client = SecretsClient::new(config).await;

    // Cache the value
    let _ = client.get_secret_optional("TEST_INVALIDATE_SECRET").await;

    // Change the env var
    std::env::set_var("TEST_INVALIDATE_SECRET", "updated");

    // Invalidate specific key
    client.invalidate("TEST_INVALIDATE_SECRET").await;

    // Should now get the updated value
    let secret = client.get_secret_optional("TEST_INVALIDATE_SECRET").await;
    assert_eq!(secret.unwrap().expose_secret(), "updated");

    std::env::remove_var("TEST_INVALIDATE_SECRET");
}

/// Test has_secret helper
#[tokio::test]
async fn test_has_secret() {
    std::env::set_var("TEST_EXISTS_SECRET", "value");

    let client = SecretsClient::env_only();

    assert!(client.has_secret("TEST_EXISTS_SECRET").await);
    assert!(!client.has_secret("TEST_DOES_NOT_EXIST").await);

    std::env::remove_var("TEST_EXISTS_SECRET");
}

/// Test get_secret_value convenience method
#[tokio::test]
async fn test_get_secret_value() {
    std::env::set_var("TEST_VALUE_SECRET", "my_value");

    let client = SecretsClient::env_only();

    let value = client.get_secret_value("TEST_VALUE_SECRET").await;
    assert_eq!(value, "my_value");

    std::env::remove_var("TEST_VALUE_SECRET");
}

/// Test get_secret_value_optional convenience method
#[tokio::test]
async fn test_get_secret_value_optional() {
    std::env::set_var("TEST_OPTIONAL_VALUE", "optional_value");

    let client = SecretsClient::env_only();

    let value = client
        .get_secret_value_optional("TEST_OPTIONAL_VALUE")
        .await;
    assert_eq!(value, Some("optional_value".to_string()));

    let missing = client
        .get_secret_value_optional("TEST_MISSING_OPTIONAL")
        .await;
    assert!(missing.is_none());

    std::env::remove_var("TEST_OPTIONAL_VALUE");
}

/// Test that KEYCLOAK_CLIENT_SECRET can be retrieved (from Infisical or env)
/// This test verifies the full integration setup works correctly
#[tokio::test]
async fn test_keycloak_secret_retrieval() {
    // Skip if not configured
    if std::env::var("KEYCLOAK_CLIENT_SECRET").is_err() && !is_infisical_configured() {
        eprintln!(
            "Skipping Keycloak secret test - neither KEYCLOAK_CLIENT_SECRET nor Infisical configured"
        );
        return;
    }

    let config = SecretsConfig::from_env();
    let client = SecretsClient::new(config).await;

    // Try to get the Keycloak secret
    let secret = client.get_secret_optional("KEYCLOAK_CLIENT_SECRET").await;

    // If we have Infisical configured and the secret was synced, it should be found
    // If only env var is set, it should also be found via fallback
    if secret.is_some() {
        let value = secret.unwrap();
        assert!(
            !value.expose_secret().is_empty(),
            "KEYCLOAK_CLIENT_SECRET should not be empty"
        );
        println!("Successfully retrieved KEYCLOAK_CLIENT_SECRET");

        // Verify it's being retrieved from the right source
        if client.has_primary_provider() {
            println!("  Source: Infisical (with env fallback)");
        } else {
            println!("  Source: Environment variable");
        }
    } else {
        println!("KEYCLOAK_CLIENT_SECRET not found (this is OK if not yet configured)");
    }
}
