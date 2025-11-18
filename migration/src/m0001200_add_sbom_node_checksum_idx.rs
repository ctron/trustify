use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(SbomNodeChecksum::Table)
                    .name(Indexes::SbomNodeChecksumValueTypeIdx.to_string())
                    .col(SbomNodeChecksum::Value)
                    .col(SbomNodeChecksum::Type)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(SbomNodeChecksum::Table)
                    .name(Indexes::SbomNodeChecksumValueTypeIdx.to_string())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Indexes {
    SbomNodeChecksumValueTypeIdx,
}

#[derive(DeriveIden)]
pub enum SbomNodeChecksum {
    Table,
    Value,
    Type,
}
