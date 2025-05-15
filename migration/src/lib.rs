pub use sea_orm_migration::prelude::*;
pub mod m20250514_153849_create_run_seeder;
mod m20250514_154701_create_users_table;
mod m20250514_194614_create_passwords_table;
mod m20250514_200035_create_user_wallets_table;
mod m20250515_133221_create_chat_rooms_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations: Vec<Box<dyn MigrationTrait>> = vec![
            Box::new(m20250514_154701_create_users_table::Migration),
            Box::new(m20250514_194614_create_passwords_table::Migration),
            Box::new(m20250514_200035_create_user_wallets_table::Migration),
            Box::new(m20250515_133221_create_chat_rooms_table::Migration),
        ];

        // Ensure the seeder migration is always the last one
        migrations.push(Box::new(m20250514_153849_create_run_seeder::Migration));

        migrations
    }
}
