use testcontainers::{
    core::{IntoContainerPort, WaitFor}, runners::AsyncRunner, GenericImage, ImageExt
};
use user_lib::{repository::{RoleRepository, UserRepository, UserRoleRepository}, user_service::UserService};
use user_lib::entities::PaginationParams;
use sqlx::migrate::Migrator;
use user_lib::util::*;

static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::test]
async fn integration_user_service_flow() {
     let image = GenericImage::new("mysql", "8")
        .with_wait_for(WaitFor::message_on_stderr("ready for connections"))
        .with_env_var("MYSQL_ROOT_PASSWORD", "password")
        .with_env_var("MYSQL_DATABASE", "testdb")
        .with_env_var("MYSQL_USER", "testuser")
        .with_env_var("MYSQL_PASSWORD", "testpass")
        .with_mapped_port(3306, 3306.tcp());

    let container = image.start().await.expect("Failed to start MySQL container");

    let port = container.get_host_port_ipv4(3306)
                        .await
                        .expect("Failed to get MySQL port");

    let db_url = format!("mysql://testuser:testpass@localhost:{}/testdb", port);

    let pool = connect_with_retry(&db_url, 10).await.expect("Failed to connect to database");
    MIGRATOR.run(&pool).await.unwrap();

    let user_repo = UserRepository::new(pool.clone());
    let role_repo = RoleRepository::new(pool.clone());
    let user_role_repo = UserRoleRepository::new(pool.clone());
    let user_service = UserService::new(user_repo, role_repo, user_role_repo);

    // Verify seeded data exists
    let seeded_roles = user_service.get_roles(PaginationParams::default()).await.unwrap();
    assert_eq!(seeded_roles.items.len(), 2, "Should have seeded admin and user roles");
    assert!(seeded_roles.items.iter().any(|r| r.name == "admin"), "Should have admin role");
    assert!(seeded_roles.items.iter().any(|r| r.name == "user"), "Should have user role");

    let seeded_users = user_service.get_users(PaginationParams::default()).await.unwrap();
    assert_eq!(seeded_users.items.len(), 1, "Should have seeded root user");
    assert!(seeded_users.items.iter().any(|u| u.keycloak_id == "root-placeholder-keycloak-id"), "Should have root user with placeholder keycloak_id");

    // Create additional users (with keycloak_id only - profile data is in Keycloak)
    let user1 = user_service.create_user("kc-alice-12345").await.unwrap();
    let user2 = user_service.create_user("kc-bob-67890").await.unwrap();
    let user3 = user_service.create_user("kc-charlie-11111").await.unwrap();

    // Create additional roles (different names from seeded ones)
    let role_editor = user_service.create_role("editor").await.unwrap();
    let _role_viewer = user_service.create_role("viewer").await.unwrap();

    // Assign users to editor role
    user_service.assign_role(user1.id, role_editor.id).await.unwrap();
    user_service.assign_role(user2.id, role_editor.id).await.unwrap();

    // Should have 4 users total (1 seeded root + 3 created)
    let list_users = user_service.get_users(PaginationParams::default()).await.unwrap();
    assert_eq!(list_users.items.len(), 4);

    // Delete user3
    user_service.delete_user(user3.id).await.unwrap();
    let deleted = user_service.get_user(user3.id).await.unwrap();
    assert!(deleted.is_none());

    // Should have 3 users remaining (1 seeded root + 2 created)
    let list_users = user_service.get_users(PaginationParams::default()).await.unwrap();
    assert_eq!(list_users.items.len(), 3);

    // Verify total roles (2 seeded + 2 created)
    let list_roles = user_service.get_roles(PaginationParams::default()).await.unwrap();
    assert_eq!(list_roles.items.len(), 4);
}
