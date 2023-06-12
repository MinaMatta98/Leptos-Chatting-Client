use crate::migrator;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement};
use sea_orm_migration::prelude::*;

#[derive(Debug)]
pub struct DbConnection {
    pub connection: DatabaseConnection,
}

#[cfg(feature = "ssr")]
impl DbConnection {
    pub async fn connect() -> DatabaseConnection {
        println!("Retrieving global database variables");
        let database_url = std::env::var("DATABASE_URL").unwrap();
        let db_name = std::env::var("DB_NAME").unwrap();
        println!("Retrieved global database variables");
        let url = format!("{}/{}", database_url, db_name);
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::task::spawn_local(async move { tx.send(Database::connect(&url).await.unwrap()) })
            .await
            .unwrap()
            .unwrap();
        rx.recv().unwrap()
    }
}

pub async fn database_run() -> Result<(), DbErr> {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(val) => val,
        Err(e) => panic!(
            "DATABASE_URL is likely not set 
        as an environmental variable. The following error has been returned: {e}"
        ),
    };

    let db_name = match std::env::var("DB_NAME") {
        Ok(val) => val,
        Err(e) => panic!(
            "DB_NAME is likely not set as an environmental variable. 
        The following error has been returned: {e}"
        ),
    };

    let db = Database::connect(database_url.clone()).await?;

    let db = &match db.get_database_backend() {
        DbBackend::MySql => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE IF NOT EXISTS `{}`;", db_name),
            ))
            .await?;

            let url = format!("{}/{}", database_url, db_name);
            Database::connect(&url).await?
        }
        DbBackend::Postgres => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("DROP DATABASE IF EXISTS \"{}\";", db_name),
            ))
            .await?;
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE \"{}\";", db_name),
            ))
            .await?;

            let url = format!("{}/{}", database_url, db_name);
            Database::connect(&url).await?
        }
        DbBackend::Sqlite => db,
    };

    let schema_manager = SchemaManager::new(db); // To investigate the schema
    migrator::Migrator::up(db, None).await?;
    // migrator::Migrator::refresh(db).await?;
    assert!(schema_manager.has_table("users").await?);
    Ok(())
}
