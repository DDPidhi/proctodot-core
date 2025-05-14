use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserWallets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserWallets::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserWallets::UserId).big_integer().not_null())
                    // Save the encrypted mnemonic in the "mnemonic" column
                    .col(ColumnDef::new(UserWallets::Mnemonic).text().not_null())
                    // Save the encrypted private key in the "private_key" column
                    .col(ColumnDef::new(UserWallets::PrivateKey).text().not_null())
                    .col(ColumnDef::new(UserWallets::PublicKey).text().not_null())
                    .col(ColumnDef::new(UserWallets::Address).string().not_null().unique_key())
                    .col(
                        ColumnDef::new(UserWallets::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UserWallets::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP"),
                    )
                    .col(ColumnDef::new(UserWallets::DeletedAt).timestamp().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(UserWallets::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
pub enum UserWallets {
    Table,
    Id,
    UserId,
    Mnemonic,
    PrivateKey,
    PublicKey,
    Address,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
