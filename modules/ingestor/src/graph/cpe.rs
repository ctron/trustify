use crate::graph::error::Error;
use crate::graph::Graph;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use sea_query::SelectStatement;
use std::fmt::{Debug, Formatter};
use tracing::instrument;
use trustify_common::{
    cpe::{Component, Component::Value, Cpe, CpeType},
    db::Transactional,
};
use trustify_entity as entity;

impl Graph {
    pub async fn get_cpe<C: Into<Cpe>, TX: AsRef<Transactional>>(
        &self,
        cpe: C,
        tx: TX,
    ) -> Result<Option<CpeContext>, Error> {
        let cpe = cpe.into();

        let mut query = entity::cpe::Entity::find();

        query = match cpe.part() {
            CpeType::Any => query.filter(entity::cpe::Column::Part.eq("*")),
            CpeType::Hardware => query.filter(entity::cpe::Column::Part.eq("h")),
            CpeType::OperatingSystem => query.filter(entity::cpe::Column::Part.eq("o")),
            CpeType::Application => query.filter(entity::cpe::Column::Part.eq("a")),
            CpeType::Empty => query.filter(entity::cpe::Column::Part.is_null()),
        };

        query = match cpe.vendor() {
            Component::Any => query.filter(entity::cpe::Column::Vendor.eq("*")),
            Component::NotApplicable => query.filter(entity::cpe::Column::Vendor.is_null()),
            Value(inner) => query.filter(entity::cpe::Column::Vendor.eq(inner)),
        };

        query = match cpe.product() {
            Component::Any => query.filter(entity::cpe::Column::Product.eq("*")),
            Component::NotApplicable => query.filter(entity::cpe::Column::Product.is_null()),
            Value(inner) => query.filter(entity::cpe::Column::Product.eq(inner)),
        };

        query = match cpe.version() {
            Component::Any => query.filter(entity::cpe::Column::Version.eq("*")),
            Component::NotApplicable => query.filter(entity::cpe::Column::Version.is_null()),
            Value(inner) => query.filter(entity::cpe::Column::Version.eq(inner)),
        };

        query = match cpe.update() {
            Component::Any => query.filter(entity::cpe::Column::Update.eq("*")),
            Component::NotApplicable => query.filter(entity::cpe::Column::Update.is_null()),
            Value(inner) => query.filter(entity::cpe::Column::Update.eq(inner)),
        };

        query = match cpe.edition() {
            Component::Any => query.filter(entity::cpe::Column::Edition.eq("*")),
            Component::NotApplicable => query.filter(entity::cpe::Column::Edition.is_null()),
            Value(inner) => query.filter(entity::cpe::Column::Edition.eq(inner)),
        };

        if let Some(found) = query.one(&self.connection(&tx)).await? {
            Ok(Some((self, found).into()))
        } else {
            Ok(None)
        }
    }

    pub async fn get_cpe_by_query<TX: AsRef<Transactional>>(
        &self,
        query: SelectStatement,
        tx: TX,
    ) -> Result<Vec<CpeContext>, Error> {
        Ok(entity::cpe::Entity::find()
            .filter(entity::cpe::Column::Id.in_subquery(query))
            .all(&self.connection(&tx))
            .await?
            .into_iter()
            .map(|cpe22| (self, cpe22).into())
            .collect())
    }

    #[instrument(skip(self, tx), err)]
    pub async fn ingest_cpe22<C, TX>(&self, cpe: C, tx: TX) -> Result<CpeContext, Error>
    where
        C: Into<Cpe> + Debug,
        TX: AsRef<Transactional>,
    {
        let cpe = cpe.into();

        if let Some(found) = self.get_cpe(cpe.clone(), &tx).await? {
            return Ok(found);
        }

        let entity: entity::cpe::ActiveModel = cpe.into();

        Ok((self, entity.insert(&self.connection(&tx)).await?).into())
    }
}

#[derive(Clone)]
pub struct CpeContext {
    pub system: Graph,
    pub cpe: entity::cpe::Model,
}

impl From<(&Graph, entity::cpe::Model)> for CpeContext {
    fn from((system, cpe22): (&Graph, entity::cpe::Model)) -> Self {
        Self {
            system: system.clone(),
            cpe: cpe22,
        }
    }
}

impl Debug for CpeContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.cpe.fmt(f)
    }
}
