use sea_orm::DatabaseConnection;
use std::error::Error;

#[async_trait::async_trait]
pub trait Placeable {
    async fn fetch_by_id(db: &DatabaseConnection, id: i64) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
    async fn get_value(&self, db: &DatabaseConnection, attr: &str) -> Option<String>;
}
