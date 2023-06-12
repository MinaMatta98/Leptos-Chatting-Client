use super::{
    m20230521_000001_create_user_table::Users, m20230606_000004_create_message_table::Message,
};
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20230606_000006_create_seen_messages_table.rs"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the UserConversation table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SeenMessages::Table)
                    .col(ColumnDef::new(SeenMessages::MessageId).integer().not_null())
                    .col(ColumnDef::new(SeenMessages::SeenId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ext_message_id")
                            .from(SeenMessages::Table, SeenMessages::MessageId)
                            .to(Message::Table, Message::MessageId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ext_seen_id")
                            .from(SeenMessages::Table, SeenMessages::SeenId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .col(SeenMessages::MessageId)
                            .col(SeenMessages::SeenId),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SeenMessages::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum SeenMessages {
    Table,
    MessageId,
    SeenId,
}
