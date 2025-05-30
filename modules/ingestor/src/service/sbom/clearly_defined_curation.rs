use crate::{
    graph::{Graph, Outcome, sbom::clearly_defined::Curation},
    model::IngestResult,
    service::Error,
};
use sea_orm::TransactionTrait;
use tracing::instrument;
use trustify_common::{hashing::Digests, id::Id};
use trustify_entity::labels::Labels;

pub struct ClearlyDefinedCurationLoader<'g> {
    graph: &'g Graph,
}

impl<'g> ClearlyDefinedCurationLoader<'g> {
    pub fn new(graph: &'g Graph) -> Self {
        Self { graph }
    }

    #[instrument(skip(self, curation), err(level=tracing::Level::INFO))]
    pub async fn load(
        &self,
        labels: Labels,
        curation: Curation,
        digests: &Digests,
    ) -> Result<IngestResult, Error> {
        let tx = self.graph.db.begin().await?;

        let sbom = match self
            .graph
            .ingest_sbom(
                labels,
                digests,
                Some(curation.document_id()),
                &curation,
                &tx,
            )
            .await?
        {
            Outcome::Existed(sbom) => sbom,
            Outcome::Added(sbom) => {
                sbom.ingest_clearly_defined_curation(curation, &tx)
                    .await
                    .map_err(Error::Generic)?;

                tx.commit().await?;

                sbom
            }
        };

        Ok(IngestResult {
            id: Id::Uuid(sbom.sbom.sbom_id),
            document_id: sbom.sbom.document_id,
            warnings: vec![],
        })
    }
}

#[cfg(test)]
mod test {
    use crate::graph::Graph;
    use crate::service::{Cache, Format, IngestorService};
    use test_context::test_context;
    use test_log::test;
    use trustify_test_context::TrustifyContext;
    use trustify_test_context::document_bytes;

    #[test_context(TrustifyContext)]
    #[test(tokio::test)]
    async fn ingest_clearly_defined(ctx: &TrustifyContext) -> Result<(), anyhow::Error> {
        let graph = Graph::new(ctx.db.clone());
        let ingestor = IngestorService::new(graph, ctx.storage.clone(), Default::default());

        let data = document_bytes("clearly-defined/chrono.yaml").await?;

        ingestor
            .ingest(
                &data,
                Format::ClearlyDefinedCuration,
                ("source", "test"),
                None,
                Cache::Skip,
            )
            .await
            .expect("must ingest");

        Ok(())
    }
}
