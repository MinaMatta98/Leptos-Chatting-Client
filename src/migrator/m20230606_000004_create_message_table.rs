use super::m20230521_000001_create_user_table::Users;
use super::m20230606_000003_create_conversation_table::Conversation;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20230606_000004_create_message_table.rs"
    }
}

#[async_trait::async_trait]
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Message table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Message::Table)
                    .col(
                        ColumnDef::new(Message::MessageId)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Message::MessageBody).string())
                    .col(ColumnDef::new(Message::MessageImage).string())
                    .col(
                        ColumnDef::new(Message::MessageCreatedAt)
                            .timestamp()
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string()),
                    )
                    .col(
                        ColumnDef::new(Message::MessageConversationId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Message::MessageSenderId)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sender_id")
                            .from(Message::Table, Message::MessageSenderId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_conversation_id")
                            .from(Message::Table, Message::MessageConversationId)
                            .to(Conversation::Table, Conversation::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Message table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Message::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Message {
    Table,
    MessageId,
    MessageBody,
    MessageImage,
    MessageCreatedAt,
    MessageConversationId,
    MessageSenderId,
}
