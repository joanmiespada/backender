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
        //.with_exposed_port(3306.tcp())
        .with_wait_for(WaitFor::message_on_stderr("ready for connections"))
        .with_env_var("MYSQL_ROOT_PASSWORD", "password")
        .with_env_var("MYSQL_DATABASE", "testdb")
        .with_env_var("MYSQL_USER", "testuser")
        .with_env_var("MYSQL_PASSWORD", "testpass")
        //.start()
        //.await
        //.expect("Failed to start MySQL container");
        .with_mapped_port(3306, 3306.tcp());

    let container = image.start().await.expect("Failed to start MySQL container");

    let port = container.get_host_port_ipv4(3306)
                        .await
                        .expect("Failed to get MySQL port");

    let db_url = format!("mysql://testuser:testpass@localhost:{}/testdb", port);

    let pool = connect_with_retry(&db_url, 10).await;
    MIGRATOR.run(&pool).await.unwrap();

    let user_repo = UserRepository::new(pool.clone());
    let role_repo = RoleRepository::new(pool.clone());
    let user_role_repo = UserRoleRepository::new(pool.clone());
    let user_service = UserService::new(user_repo, role_repo, user_role_repo);

    // Create users
    let user1 = user_service.create_user("Alice", "alice@example.com").await.unwrap();
    let user2 = user_service.create_user("Bob", "bob@example.com").await.unwrap();
    let user3 = user_service.create_user("Charlie", "charlie@example.com").await.unwrap();

    // Create roles
    let role1 = user_service.create_role("Admin").await.unwrap();
    let _role2 = user_service.create_role("User").await.unwrap();

    // Assign users
    user_service.assign_role(user1.id, role1.id).await.unwrap();
    user_service.assign_role(user2.id, role1.id).await.unwrap();

    let list_users = user_service.get_users(PaginationParams::default()).await.unwrap();
    assert_eq!(list_users.items.len(), 3);

    // Delete user3
    user_service.delete_user(user3.id).await.unwrap();
    let deleted = user_service.get_user(user3.id).await.unwrap();
    assert!(deleted.is_none());

    //let list_users = user_service.get_users().await.unwrap();
    //assert_eq!(list_users.len(), 2);

}