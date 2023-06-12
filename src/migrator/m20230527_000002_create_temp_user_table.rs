use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20230527_000002_create_temp_user_table.rs"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Bakery table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TempUsers::Table)
                    .col(
                        ColumnDef::new(TempUsers::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TempUsers::FirstName).string().not_null())
                    .col(ColumnDef::new(TempUsers::LastName).string().not_null())
                    .col(ColumnDef::new(TempUsers::Email).string().not_null())
                    .col(ColumnDef::new(TempUsers::PhoneNumber).big_integer().not_null())
                    .col(ColumnDef::new(TempUsers::Password).string().not_null())
                    .col(ColumnDef::new(TempUsers::Verification).string().not_null())
                    .col(ColumnDef::new(TempUsers::Time).timestamp().not_null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TempUsers::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum TempUsers {
    Table,
    Id,
    FirstName,
    LastName,
    Email,
    PhoneNumber,
    Password,
    Verification,
    Time,
}
