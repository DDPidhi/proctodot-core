use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the users table with a custom ENUM for the 'type' column in MariaDB
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::Chain).string().not_null())
                    .col(ColumnDef::new(Users::Email).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::FirstName).string().not_null())
                    .col(ColumnDef::new(Users::LastName).string().not_null())
                    .col(ColumnDef::new(Users::Phone).string().not_null())
                    .col(
                        ColumnDef::new(Users::Type)
                            .enumeration(Users::Type, vec![UserTypeEnum::Member, UserTypeEnum::Proctor, UserTypeEnum::Admin])
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Users::DeletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the users table
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
    Chain,
    Email,
    FirstName,
    LastName,
    Phone,
    Type,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

// Define an enum for the 'type' column values
#[derive(Iden)]
pub enum UserTypeEnum {
    #[iden = "member"]
    Member,
    #[iden = "proctor"]
    Proctor,
    #[iden = "admin"]
    Admin,
}