use sea_orm_migration::{MigrationTrait, MigratorTrait};

pub struct Migrator;
mod m20230521_000001_create_user_table;
mod m20230527_000002_create_temp_user_table;
mod m20230606_000003_create_conversation_table;
mod m20230606_000004_create_message_table;
mod m20230606_000005_create_user_conversation_table;
mod m20230606_000006_create_seen_messages_table;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230521_000001_create_user_table::Migration),
            Box::new(m20230527_000002_create_temp_user_table::Migration),
            Box::new(m20230606_000003_create_conversation_table::Migration),
            Box::new(m20230606_000004_create_message_table::Migration),
            Box::new(m20230606_000005_create_user_conversation_table::Migration),
            Box::new(m20230606_000006_create_seen_messages_table::Migration)
        ]
    }
}
