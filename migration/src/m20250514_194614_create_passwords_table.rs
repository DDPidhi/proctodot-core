use sea_orm_migration::prelude::*;
use crate::m20250514_154701_create_users_table::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Passwords::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Passwords::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Passwords::UserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Passwords::PasswordHash)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Passwords::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_password_user")
                            .from(Passwords::Table, Passwords::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Passwords::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Passwords {
    Table,
    Id,
    UserId,
    PasswordHash,
    CreatedAt,
}
