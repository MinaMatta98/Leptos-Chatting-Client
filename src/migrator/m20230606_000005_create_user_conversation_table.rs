use super::m20230521_000001_create_user_table::Users;
use super::m20230606_000003_create_conversation_table::Conversation;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20230606_000005_create_user_conversation_table.rs"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the UserConversation table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserConversation::Table)
                    .col(
                        ColumnDef::new(UserConversation::UserIds)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserConversation::ConversationId)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_id")
                            .from(UserConversation::Table, UserConversation::UserIds)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_conversation_id")
                            .from(UserConversation::Table, UserConversation::ConversationId)
                            .to(Conversation::Table, Conversation::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .col(UserConversation::ConversationId)
                            .col(UserConversation::UserIds),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserConversation::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum UserConversation {
    Table,
    UserIds,
    ConversationId,
}
