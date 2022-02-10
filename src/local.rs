use std::ops::Deref;
use std::path::PathBuf;
use std::time::Duration;

use async_once::AsyncOnce;
use lazy_static::lazy_static;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use sea_orm::{ConnectionTrait, DatabaseConnection, Schema, Statement};
use tokio::sync::Mutex;

use crate::entities::*;
use crate::types::*;

pub(crate) fn join_paths(paths: Vec<&str>) -> String {
    match paths.len() {
        0 => String::default(),
        _ => {
            let mut path: PathBuf = PathBuf::new();
            for x in 0..paths.len() {
                path = path.join(paths[x]);
            }
            return path.to_str().unwrap().to_string();
        }
    }
}

#[cfg(target_os = "macos")]
fn cfg_local_dir() -> String {
    join_paths(vec![
        dirs::home_dir().unwrap().to_str().unwrap(),
        "Library",
        "Application Support",
        "bili-cli",
    ])
}

#[cfg(target_os = "windows")]
fn cfg_local_dir() -> String {
    join_paths(vec![
        dirs::home_dir().unwrap().to_str().unwrap(),
        "AppData",
        "Roaming",
        "bili-cli",
    ])
}

#[cfg(target_os = "linux")]
fn cfg_local_dir() -> String {
    join_paths(vec![
        dirs::home_dir().unwrap().to_str().unwrap(),
        ".bili-cli",
    ])
}

#[cfg(target_os = "macos")]
pub(crate) fn template_dir() -> String {
    "/tmp".to_owned()
}

#[cfg(target_os = "linux")]
pub(crate) fn template_dir() -> String {
    "/tmp".to_owned()
}

fn init_dir() {
    std::fs::create_dir_all(cfg_local_dir()).unwrap();
}

pub(crate) async fn connect_db(path: &str) -> DatabaseConnection {
    let url = format!("sqlite:{}?mode=rwc", path);
    let mut opt = sea_orm::ConnectOptions::new(url);
    opt.max_connections(20)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true);
    sea_orm::Database::connect(opt).await.unwrap()
}

pub(crate) async fn create_table_if_not_exists<E>(db: &DatabaseConnection, entity: E)
where
    E: EntityTrait,
{
    if !has_table(db, entity.table_name()).await {
        create_table(db, entity).await;
    };
}

async fn has_table(db: &DatabaseConnection, table_name: &str) -> bool {
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            "SELECT COUNT(*) AS c FROM sqlite_master WHERE type='table' AND name='{}';",
            table_name,
        ),
    );
    let rsp = db.query_one(stmt).await.unwrap().unwrap();
    let count: i32 = rsp.try_get("", "c").unwrap();
    count > 0
}

async fn create_table<E>(db: &DatabaseConnection, entity: E)
where
    E: EntityTrait,
{
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    let stmt = &schema.create_table_from_entity(entity);
    let stmt = builder.build(stmt);
    db.execute(stmt).await.unwrap();
}
pub(crate) async fn index_exists(
    db: &DatabaseConnection,
    table_name: &str,
    index_name: &str,
) -> bool {
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            "select COUNT(*) AS c from sqlite_master where type='index' AND tbl_name='{}' AND name='{}';",
            table_name, index_name,
        ),
    );
    db.query_one(stmt)
        .await
        .unwrap()
        .unwrap()
        .try_get::<i32>("", "c")
        .unwrap()
        > 0
}

pub(crate) async fn create_index(
    db: &DatabaseConnection,
    table_name: &str,
    columns: Vec<&str>,
    index_name: &str,
) {
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            "CREATE INDEX {} ON {}({});",
            index_name,
            table_name,
            columns.join(","),
        ),
    );
    db.execute(stmt).await.unwrap();
}

lazy_static! {
    pub(crate) static ref PROPERTY_DB: AsyncOnce<Mutex<DatabaseConnection>> =
        AsyncOnce::new(async {
            init_dir();
            let db =
                connect_db(join_paths(vec![cfg_local_dir().as_str(), "properties.db"]).as_str())
                    .await;
            create_table_if_not_exists(&db, property::Entity).await;
            property::init_indexes(&db).await;
            Mutex::<DatabaseConnection>::new(db)
        });
}

pub(crate) async fn load_property_from_db(db: &DatabaseConnection, k: String) -> Result<String> {
    let in_db = property::Entity::find_by_id(k.clone())
        .one(db.deref())
        .await?;
    Ok(match in_db {
        Some(in_db) => in_db.v,
        None => String::default(),
    })
}

pub(crate) async fn load_property(k: String) -> Result<String> {
    load_property_from_db(PROPERTY_DB.get().await.lock().await.deref(), k).await
}

pub(crate) async fn save_property_from_db(
    db: &DatabaseConnection,
    k: String,
    v: String,
) -> Result<()> {
    let in_db = property::Entity::find_by_id(k.clone())
        .one(db.deref())
        .await?;
    match in_db {
        Some(in_db) => {
            let mut data: property::ActiveModel = in_db.into();
            data.k = Set(k.clone());
            data.v = Set(v.clone());
            data.update(db.deref()).await?;
        }
        None => {
            let insert = property::ActiveModel {
                k: Set(k.clone()),
                v: Set(v.clone()),
                ..Default::default()
            };
            insert.insert(db.deref()).await?;
        }
    };
    Ok(())
}

pub(crate) async fn save_property(k: String, v: String) -> Result<()> {
    save_property_from_db(PROPERTY_DB.get().await.lock().await.deref(), k, v).await
}
