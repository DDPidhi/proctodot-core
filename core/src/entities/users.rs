use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter};
use crate::traits::placeable::Placeable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub chain: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub r#type: String,

    #[serde(skip_serializing)]
    pub created_at: DateTimeUtc,
    #[serde(skip_serializing)]
    pub updated_at: DateTimeUtc,
    #[serde(skip_serializing)]
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        unimplemented!()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[async_trait]
impl Placeable for Model {
    async fn fetch_by_id(db: &DatabaseConnection, id: i64) -> Result<Self, Box<dyn std::error::Error>> {
        let user = Entity::find_by_id(id as i32)
            .one(db)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?; // Convert `DbErr` to `Box<dyn Error>`
        user.ok_or_else(|| "User not found".into())
    }

    async fn get_value(&self, db: &DatabaseConnection, attr: &str) -> Option<String> {
        match attr {
            "first_name" => Some(self.first_name.clone()),
            "last_name" => Some(self.last_name.clone()),
            "email" => Some(self.email.clone()),
            "full_name" => Some(self.get_full_name()),
            "phone" => Some(self.phone.clone()),
            "chain" => Some(self.chain.clone()),
            _ => None,
        }
    }
}

impl Model {
    pub fn get_full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}