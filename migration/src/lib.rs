pub use sea_orm_migration::prelude::*;

mod data;
pub use crate::data::{MigrationTraitWithData, MigrationWithData, Options, SchemaDataManager};

mod m0000010_init;
mod m0000020_add_sbom_group;
mod m0000030_perf_adv_vuln;
mod m0000040_create_license_export;
mod m0000050_perf_adv_vuln2;
mod m0000060_perf_adv_vuln3;
mod m0000070_perf_adv_vuln4;
mod m0000080_get_purl_refactor;
mod m0000090_release_perf;
mod m0000100_perf_adv_vuln5;
mod m0000970_alter_importer_add_heartbeat;
mod m0000980_get_purl_fix;
mod m0000990_sbom_add_suppliers;
mod m0001000_sbom_non_null_suppliers;
mod m0001010_alter_mavenver_cmp;
mod m0001020_alter_pythonver_cmp;
mod m0001030_perf_adv_gin_index;
mod m0001040_alter_pythonver_cmp;
mod m0001050_foreign_key_cascade;
mod m0001060_advisory_vulnerability_indexes;
mod m0001070_vulnerability_scores;
mod m0001100_remove_get_purl;
mod m0001110_sbom_node_checksum_indexes;
mod m0001120_sbom_external_node_indexes;
mod m0001130_gover_cmp;
mod m0001140_expand_spdx_licenses_function;
mod m0001150_case_license_text_sbom_id_function;
mod m0002000_example_data_migration;

#[derive(Default)]
pub struct Migrations {
    all: Vec<Migration>,
}

impl IntoIterator for Migrations {
    type Item = Migration;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.all.into_iter()
    }
}

pub enum Migration {
    Normal(Box<dyn MigrationTrait>),
    Data(Box<dyn MigrationTraitWithData>),
}

impl Migrations {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn normal(mut self, migration: impl MigrationTrait + 'static) -> Self {
        self.all.push(Migration::Normal(Box::new(migration)));
        self
    }

    pub fn data(mut self, migration: impl MigrationTraitWithData + 'static) -> Self {
        self.all.push(Migration::Data(Box::new(migration)));
        self
    }
}

pub struct Migrator;

impl Migrator {
    fn migrations() -> Migrations {
        Migrations::new()
            .normal(m0000010_init::Migration)
            .normal(m0000020_add_sbom_group::Migration)
            .normal(m0000030_perf_adv_vuln::Migration)
            .normal(m0000040_create_license_export::Migration)
            .normal(m0000050_perf_adv_vuln2::Migration)
            .normal(m0000060_perf_adv_vuln3::Migration)
            .normal(m0000070_perf_adv_vuln4::Migration)
            .normal(m0000080_get_purl_refactor::Migration)
            .normal(m0000090_release_perf::Migration)
            .normal(m0000100_perf_adv_vuln5::Migration)
            .normal(m0000970_alter_importer_add_heartbeat::Migration)
            .normal(m0000980_get_purl_fix::Migration)
            .normal(m0000990_sbom_add_suppliers::Migration)
            .normal(m0001000_sbom_non_null_suppliers::Migration)
            .normal(m0001010_alter_mavenver_cmp::Migration)
            .normal(m0001020_alter_pythonver_cmp::Migration)
            .normal(m0001030_perf_adv_gin_index::Migration)
            .normal(m0001040_alter_pythonver_cmp::Migration)
            .normal(m0001050_foreign_key_cascade::Migration)
            .normal(m0001060_advisory_vulnerability_indexes::Migration)
            .normal(m0001070_vulnerability_scores::Migration)
            .normal(m0001100_remove_get_purl::Migration)
            .normal(m0001110_sbom_node_checksum_indexes::Migration)
            .normal(m0001120_sbom_external_node_indexes::Migration)
            .normal(m0001130_gover_cmp::Migration)
            .data(m0002000_example_data_migration::Migration)
    }

    pub fn data_migrations() -> Vec<Box<dyn MigrationTraitWithData>> {
        Self::migrations()
            .into_iter()
            .filter_map(|migration| match migration {
                Migration::Normal(_) => None,
                Migration::Data(migration) => Some(migration),
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        Self::migrations()
            .into_iter()
            .map(|migration| match migration {
                Migration::Normal(migration) => migration,
                Migration::Data(migration) => Box::new(MigrationWithData::new(migration)),
            })
            .collect()
    }
}

pub struct Now;

impl Iden for Now {
    #[allow(clippy::unwrap_used)]
    fn unquoted(&self, s: &mut dyn Write) {
        write!(s, "now").unwrap()
    }
}

pub struct UuidV4;

impl Iden for UuidV4 {
    #[allow(clippy::unwrap_used)]
    fn unquoted(&self, s: &mut dyn Write) {
        write!(s, "gen_random_uuid").unwrap()
    }
}
