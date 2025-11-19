pub use sea_orm_migration::prelude::*;

mod m20251021_143555_create_user_table;
mod m20251021_160348_create_instance_settings_table;
mod m20251110_102409_create_calendar_table;
mod m20251114_114030_add_display_name;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251021_143555_create_user_table::Migration),
            Box::new(m20251021_160348_create_instance_settings_table::Migration),
            Box::new(m20251110_102409_create_calendar_table::Migration),
            Box::new(m20251114_114030_add_display_name::Migration),
        ]
    }
}
