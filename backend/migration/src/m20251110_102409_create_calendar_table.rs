use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Calendar::Table)
                    .if_not_exists()
                    .col(pk_uuid(Calendar::Id))
                    .col(uuid(Calendar::Owner))
                    .col(string(Calendar::Title))
                    .col(string(Calendar::Colour))
                    .col(
                        timestamp_with_time_zone(Calendar::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Calendar::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "
                CREATE OR REPLACE FUNCTION upd_timestamp() RETURNS TRIGGER
                LANGUAGE plpgsql
                AS
                $$
                BEGIN
                    NEW.updated_at = CURRENT_TIMESTAMP;
                    RETURN NEW;
                END;
                $$;

                CREATE TRIGGER t_upd_timestamp
                  BEFORE UPDATE
                  ON calendar
                  FOR EACH ROW
                  EXECUTE PROCEDURE upd_timestamp();
            ",
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("FK_calendar_user")
                    .from(Calendar::Table, Calendar::Owner)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade) // NOTE(Julius): Scawy
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "
            DROP TRIGGER t_upd_timestamp ON calendar;
            DROP FUNCTION upd_timestamp();
            ",
            )
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .table(Calendar::Table)
                    .name("FK_calendar_user")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Calendar::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Calendar {
    Table,
    Id,
    Title,
    Owner,
    Colour,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}
