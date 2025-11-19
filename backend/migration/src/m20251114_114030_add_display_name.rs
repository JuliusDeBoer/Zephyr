use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(User::Table)
                    .add_column(string(User::DisplayName))
                    .take(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(User::Table)
                    .drop_column(User::DisplayName)
                    .take(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    DisplayName,
}
