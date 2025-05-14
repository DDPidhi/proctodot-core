use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use crate::encryption::encryption::Encryptor;

#[derive(Clone, Default, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_wallets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub user_id: i64,

    #[sea_orm(column_type = "Text", column_name = "mnemonic")]
    pub encrypted_mnemonic: String,

    #[sea_orm(column_type = "Text", column_name = "private_key")]
    pub encrypted_private_key: String,

    #[sea_orm(column_type = "Text")]
    pub public_key: String,

    #[sea_orm(unique)]
    pub address: String,

    #[serde(skip_serializing)]
    pub created_at: DateTimeUtc,

    #[serde(skip_serializing)]
    pub updated_at: DateTimeUtc,

    #[serde(skip_serializing)]
    pub deleted_at: Option<DateTimeUtc>,
}

impl Model {
    /// Encrypt the mnemonic before saving to the database
    pub fn set_mnemonic(&mut self, mnemonic: &str) -> Result<(), &'static str> {
        let encrypted_data = Encryptor::encrypt_data(mnemonic)?;
        self.encrypted_mnemonic = encrypted_data;
        Ok(())
    }

    /// Decrypt the mnemonic after retrieving it from the database
    pub fn get_mnemonic(&self) -> Result<String, &'static str> {
        Encryptor::decrypt_data(&self.encrypted_mnemonic)
    }

    /// Encrypt the private key before saving to the database
    pub fn set_private_key(&mut self, private_key: &str) -> Result<(), &'static str> {
        let encrypted_data = Encryptor::encrypt_data(private_key)?;
        self.encrypted_private_key = encrypted_data;
        Ok(())
    }

    /// Decrypt the private key after retrieving it from the database
    pub fn get_private_key(&self) -> Result<String, &'static str> {
        Encryptor::decrypt_data(&self.encrypted_private_key)
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        unimplemented!()
    }
}

impl ActiveModelBehavior for crate::entities::user_wallet::ActiveModel {}