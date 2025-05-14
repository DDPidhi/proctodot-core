use std::fs;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Load SQL from file
        let sql_file_path = "./migration/seeder.sql";
        let sql_content = fs::read_to_string(sql_file_path)
            .map_err(|err| DbErr::Custom(format!("Failed to read SQL file: {}", err)))?;

        // Execute raw SQL
        let conn = manager.get_connection();
        conn.execute_unprepared(&sql_content).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}