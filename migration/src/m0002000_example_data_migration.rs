use crate::{
    data::{MigrationTraitWithData, Sbom as SbomDoc, SchemaDataManager},
    sbom,
};
use sea_orm::{ActiveModelTrait, IntoActiveModel, Set};
use sea_orm_migration::prelude::*;
use trustify_common::advisory::cyclonedx::extract_properties_json;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTraitWithData for Migration {
    async fn up(&self, manager: &SchemaDataManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Sbom::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Sbom::Properties)
                            .json()
                            .default(serde_json::Value::Null)
                            .to_owned(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Sbom::Table)
                    .modify_column(ColumnDef::new(Sbom::Properties).not_null().to_owned())
                    .to_owned(),
            )
            .await?;

        manager
            .process(
                self,
                sbom!(async |sbom, model, tx| {
                    let mut model = model.into_active_model();
                    match sbom {
                        SbomDoc::CycloneDx(sbom) => {
                            model.properties = Set(extract_properties_json(&sbom));
                        }
                        SbomDoc::Spdx(_sbom) => {
                            model.properties = Set(serde_json::Value::Object(Default::default()));
                        }
                    }

                    model.save(tx).await?;

                    Ok(())
                }),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaDataManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Sbom::Table)
                    .drop_column(Sbom::Properties)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Sbom {
    Table,
    Properties,
}
