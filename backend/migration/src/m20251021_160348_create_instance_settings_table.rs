use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(InstanceSetting::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InstanceSetting::Key)
                            .string()
                            .primary_key()
                            .take(),
                    )
                    .col(string(InstanceSetting::Value))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(InstanceSetting::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum InstanceSetting {
    Table,
    Key,
    Value,
}
