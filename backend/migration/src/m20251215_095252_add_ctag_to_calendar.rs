use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(Calendar::Table)
                    .add_column(uuid(Calendar::CTag))
                    .take(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(Calendar::Table)
                    .drop_column(Calendar::CTag)
                    .take(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Calendar {
    Table,
    CTag,
}
