use cubelit_core::db::{models::Cubelit, queries, run_migrations};
use sqlx::sqlite::SqlitePoolOptions;

async fn setup_db() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("failed to open in-memory SQLite");
    run_migrations(&pool).await.expect("migrations failed");
    pool
}

fn make_cubelit(id: &str) -> Cubelit {
    Cubelit {
        id: id.to_string(),
        name: format!("Test Server {}", id),
        game: "minecraft".to_string(),
        recipe_id: "minecraft-java".to_string(),
        docker_image: "itzg/minecraft-server".to_string(),
        container_id: None,
        status: "created".to_string(),
        port_mappings: "{}".to_string(),
        environment: "{}".to_string(),
        volume_path: "/tmp/data".to_string(),
        container_mount_path: "/data".to_string(),
        sidecar_container_id: None,
        sidecar_image: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
    }
}

#[tokio::test]
async fn insert_and_get() {
    let db = setup_db().await;
    let c = make_cubelit("abc123");
    queries::insert_cubelit(&db, &c).await.unwrap();

    let fetched = queries::get_cubelit(&db, "abc123").await.unwrap();
    assert_eq!(fetched.id, "abc123");
    assert_eq!(fetched.name, "Test Server abc123");
    assert_eq!(fetched.status, "created");
}

#[tokio::test]
async fn get_not_found_returns_error() {
    let db = setup_db().await;
    let err = queries::get_cubelit(&db, "doesnotexist").await.unwrap_err();
    assert!(matches!(err, cubelit_core::error::CoreError::NotFound(_)));
}

#[tokio::test]
async fn list_returns_all_in_order() {
    let db = setup_db().await;
    let mut c1 = make_cubelit("aaa");
    c1.created_at = "2026-01-01T00:00:00Z".to_string();
    let mut c2 = make_cubelit("bbb");
    c2.created_at = "2026-01-02T00:00:00Z".to_string();
    queries::insert_cubelit(&db, &c1).await.unwrap();
    queries::insert_cubelit(&db, &c2).await.unwrap();

    let list = queries::list_cubelits(&db).await.unwrap();
    assert_eq!(list.len(), 2);
    // ORDER BY created_at DESC — newest first
    assert_eq!(list[0].id, "bbb");
    assert_eq!(list[1].id, "aaa");
}

#[tokio::test]
async fn update_status_without_container_id() {
    let db = setup_db().await;
    let c = make_cubelit("upd1");
    queries::insert_cubelit(&db, &c).await.unwrap();
    queries::update_cubelit_status(&db, "upd1", "running", None)
        .await
        .unwrap();
    let fetched = queries::get_cubelit(&db, "upd1").await.unwrap();
    assert_eq!(fetched.status, "running");
    assert_eq!(fetched.container_id, None);
}

#[tokio::test]
async fn update_status_with_container_id() {
    let db = setup_db().await;
    let c = make_cubelit("upd2");
    queries::insert_cubelit(&db, &c).await.unwrap();
    queries::update_cubelit_status(&db, "upd2", "running", Some("cid-xyz"))
        .await
        .unwrap();
    let fetched = queries::get_cubelit(&db, "upd2").await.unwrap();
    assert_eq!(fetched.status, "running");
    assert_eq!(fetched.container_id.as_deref(), Some("cid-xyz"));
}

#[tokio::test]
async fn update_name() {
    let db = setup_db().await;
    let c = make_cubelit("nm1");
    queries::insert_cubelit(&db, &c).await.unwrap();
    queries::update_cubelit_name(&db, "nm1", "Renamed Server")
        .await
        .unwrap();
    let fetched = queries::get_cubelit(&db, "nm1").await.unwrap();
    assert_eq!(fetched.name, "Renamed Server");
}

#[tokio::test]
async fn update_environment() {
    let db = setup_db().await;
    let c = make_cubelit("env1");
    queries::insert_cubelit(&db, &c).await.unwrap();
    queries::update_cubelit_environment(&db, "env1", r#"{"MEMORY":"4G"}"#)
        .await
        .unwrap();
    let fetched = queries::get_cubelit(&db, "env1").await.unwrap();
    assert_eq!(fetched.environment, r#"{"MEMORY":"4G"}"#);
}

#[tokio::test]
async fn update_sidecar() {
    let db = setup_db().await;
    let c = make_cubelit("sc1");
    queries::insert_cubelit(&db, &c).await.unwrap();
    queries::update_cubelit_sidecar(&db, "sc1", "mariadb-cid", "mariadb:11")
        .await
        .unwrap();
    let fetched = queries::get_cubelit(&db, "sc1").await.unwrap();
    assert_eq!(fetched.sidecar_container_id.as_deref(), Some("mariadb-cid"));
    assert_eq!(fetched.sidecar_image.as_deref(), Some("mariadb:11"));
}

#[tokio::test]
async fn delete_removes_record() {
    let db = setup_db().await;
    let c = make_cubelit("del1");
    queries::insert_cubelit(&db, &c).await.unwrap();
    queries::delete_cubelit(&db, "del1").await.unwrap();
    let err = queries::get_cubelit(&db, "del1").await.unwrap_err();
    assert!(matches!(err, cubelit_core::error::CoreError::NotFound(_)));
}
