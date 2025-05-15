use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Your chat_rooms model definition
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "chat_rooms")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub room_id: String,  // Primary Key
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        unimplemented!()
    }
}

impl ActiveModelBehavior for ActiveModel {}