use crate::graph::advisory::advisory_vulnerability::{Version, VersionInfo, VersionSpec};
use crate::model::IngestResult;
use crate::{
    graph::{
        advisory::{AdvisoryInformation, AdvisoryVulnerabilityInformation},
        Graph,
    },
    service::{advisory::osv::translate, Error},
};
use osv::schema::{Event, ReferenceType, SeverityType, Vulnerability};
use std::{io::Read, str::FromStr, sync::OnceLock};
use trustify_common::hashing::Digests;
use trustify_common::id::Id;
use trustify_common::{purl::Purl, time::ChronoExt};
use trustify_cvss::cvss3::Cvss3Base;
use trustify_entity::labels::Labels;

pub struct OsvLoader<'g> {
    graph: &'g Graph,
}

impl<'g> OsvLoader<'g> {
    pub fn new(graph: &'g Graph) -> Self {
        Self { graph }
    }

    pub async fn load<R: Read>(
        &self,
        labels: impl Into<Labels>,
        issuer: Option<String>,
        record: R,
        digests: &Digests,
    ) -> Result<IngestResult, Error> {
        let osv: Vulnerability = serde_json::from_reader(record)?;

        let issuer = issuer.or(detect_organization(&osv));

        let tx = self.graph.transaction().await?;

        let cve_ids = osv.aliases.iter().flat_map(|aliases| {
            aliases
                .iter()
                .filter(|e| e.starts_with("CVE-"))
                .cloned()
                .collect::<Vec<_>>()
        });

        let information = AdvisoryInformation {
            title: osv.summary.clone(),
            issuer,
            published: Some(osv.published.into_time()),
            modified: Some(osv.modified.into_time()),
            withdrawn: osv.withdrawn.map(ChronoExt::into_time),
        };
        let advisory = self
            .graph
            .ingest_advisory(&osv.id, labels, digests, information, &tx)
            .await?;

        if let Some(withdrawn) = osv.withdrawn {
            advisory
                .set_withdrawn_at(withdrawn.into_time(), &tx)
                .await?;
        }

        for cve_id in cve_ids {
            let advisory_vuln = advisory
                .link_to_vulnerability(
                    &cve_id,
                    Some(AdvisoryVulnerabilityInformation {
                        title: osv.summary.clone(),
                        summary: osv.summary.clone(),
                        description: osv.details.clone(),
                        discovery_date: None,
                        release_date: None,
                        cwe: None,
                    }),
                    &tx,
                )
                .await?;

            for severity in osv.severity.iter().flatten() {
                if matches!(severity.severity_type, SeverityType::CVSSv3) {
                    match Cvss3Base::from_str(&severity.score) {
                        Ok(cvss3) => {
                            advisory_vuln.ingest_cvss3_score(cvss3, &tx).await?;
                        }
                        Err(err) => {
                            log::warn!("Unable to parse CVSS3: {:#?}", err);
                        }
                    }
                }
            }

            for affected in &osv.affected {
                if let Some(package) = &affected.package {
                    let mut purls = vec![];

                    purls.extend(translate::to_purl(package).map(Purl::from));

                    if let Some(purl) = &package.purl {
                        purls.extend(Purl::from_str(purl).ok());
                    }

                    for purl in purls {
                        for range in affected.ranges.iter().flatten() {
                            let parsed_range = events_to_range(&range.events);
                            match &parsed_range {
                                (Some(start), None) => {
                                    advisory_vuln
                                        .ingest_package_status(
                                            &purl,
                                            "affected",
                                            VersionInfo {
                                                // TODO detect better version scheme
                                                scheme: "semver".to_string(),
                                                spec: VersionSpec::Range(
                                                    Version::Inclusive(start.clone()),
                                                    Version::Unbounded,
                                                ),
                                            },
                                            &tx,
                                        )
                                        .await?
                                }
                                (None, Some(end)) => {
                                    advisory_vuln
                                        .ingest_package_status(
                                            &purl,
                                            "affected",
                                            VersionInfo {
                                                // TODO detect better version scheme
                                                scheme: "semver".to_string(),
                                                spec: VersionSpec::Range(
                                                    Version::Unbounded,
                                                    Version::Exclusive(end.clone()),
                                                ),
                                            },
                                            &tx,
                                        )
                                        .await?
                                }
                                (Some(start), Some(end)) => {
                                    advisory_vuln
                                        .ingest_package_status(
                                            &purl,
                                            "affected",
                                            VersionInfo {
                                                // TODO detect better version scheme
                                                scheme: "semver".to_string(),
                                                spec: VersionSpec::Range(
                                                    Version::Inclusive(start.clone()),
                                                    Version::Exclusive(end.clone()),
                                                ),
                                            },
                                            &tx,
                                        )
                                        .await?
                                }
                                _ => { /* what? */ }
                            }

                            if let (_, Some(fixed)) = &parsed_range {
                                advisory_vuln
                                    .ingest_package_status(
                                        &purl,
                                        "fixed",
                                        VersionInfo {
                                            // TODO detect better version scheme
                                            scheme: "semver".to_string(),
                                            spec: VersionSpec::Exact(fixed.clone()),
                                        },
                                        &tx,
                                    )
                                    .await?
                            }
                        }
                    }
                }
            }
        }

        tx.commit().await?;

        Ok(IngestResult {
            id: Id::Uuid(advisory.advisory.id),
            document_id: osv.id,
        })
    }
}

fn detect_organization(osv: &Vulnerability) -> Option<String> {
    if let Some(references) = &osv.references {
        let advisory_location = references
            .iter()
            .find(|reference| matches!(reference.reference_type, ReferenceType::Advisory));

        if let Some(advisory_location) = advisory_location {
            let url = &advisory_location.url;
            return get_well_known_prefixes().detect(url);
        }
    }
    None
}

struct PrefixMatcher {
    prefixes: Vec<PrefixMapping>,
}

impl PrefixMatcher {
    fn new() -> Self {
        Self { prefixes: vec![] }
    }

    fn add(&mut self, prefix: impl Into<String>, name: impl Into<String>) {
        self.prefixes.push(PrefixMapping {
            prefix: prefix.into(),
            name: name.into(),
        })
    }

    fn detect(&self, input: &str) -> Option<String> {
        self.prefixes
            .iter()
            .find(|each| input.starts_with(&each.prefix))
            .map(|inner| inner.name.clone())
    }
}

struct PrefixMapping {
    prefix: String,
    name: String,
}

fn get_well_known_prefixes() -> &'static PrefixMatcher {
    WELL_KNOWN_PREFIXES.get_or_init(|| {
        let mut matcher = PrefixMatcher::new();

        matcher.add(
            "https://rustsec.org/advisories/RUSTSEC",
            "Rust Security Advisory Database",
        );

        matcher
    })
}

static WELL_KNOWN_PREFIXES: OnceLock<PrefixMatcher> = OnceLock::new();

fn events_to_range(events: &[Event]) -> (Option<String>, Option<String>) {
    let start = events.iter().find_map(|e| {
        if let Event::Introduced(version) = e {
            Some(version.clone())
        } else {
            None
        }
    });

    let end = events.iter().find_map(|e| {
        if let Event::Fixed(version) = e {
            Some(version.clone())
        } else {
            None
        }
    });

    (start, end)
}

#[cfg(test)]
mod test {
    use hex::ToHex;
    use test_context::test_context;
    use test_log::test;
    use trustify_common::db::test::TrustifyContext;

    use crate::graph::Graph;
    use trustify_common::db::Transactional;
    use trustify_common::hashing::Digests;

    use crate::service::advisory::osv::loader::OsvLoader;

    #[test_context(TrustifyContext, skip_teardown)]
    #[test(tokio::test)]
    async fn loader(ctx: TrustifyContext) -> Result<(), anyhow::Error> {
        let db = ctx.db;
        let graph = Graph::new(db);

        let data = include_bytes!("../../../../../../etc/test-data/osv/RUSTSEC-2021-0079.json");
        let digests = Digests::digest(data);

        let loaded_vulnerability = graph
            .get_vulnerability("CVE-2021-32714", Transactional::None)
            .await?;
        assert!(loaded_vulnerability.is_none());

        let loaded_advisory = graph
            .get_advisory_by_digest(&digests.sha256.encode_hex::<String>(), Transactional::None)
            .await?;
        assert!(loaded_advisory.is_none());

        let loader = OsvLoader::new(&graph);
        loader
            .load(
                ("file", "RUSTSEC-2021-0079.json"),
                None,
                &data[..],
                &digests,
            )
            .await?;

        let loaded_vulnerability = graph
            .get_vulnerability("CVE-2021-32714", Transactional::None)
            .await?;
        assert!(loaded_vulnerability.is_some());

        let loaded_advisory = graph
            .get_advisory_by_digest(&digests.sha256.encode_hex::<String>(), Transactional::None)
            .await?;
        assert!(loaded_advisory.is_some());

        let loaded_advisory = loaded_advisory.unwrap();

        assert!(loaded_advisory.advisory.issuer_id.is_some());

        let loaded_advisory_vulnerabilities = loaded_advisory.vulnerabilities(()).await?;
        assert_eq!(1, loaded_advisory_vulnerabilities.len());
        let _loaded_advisory_vulnerability = &loaded_advisory_vulnerabilities[0];

        /*
        let affected_assertions = loaded_advisory_vulnerability
            .affected_assertions(())
            .await?;
        assert_eq!(1, affected_assertions.assertions.len());

        let affected_assertion = affected_assertions.assertions.get("pkg://cargo/hyper");
        assert!(affected_assertion.is_some());

        let affected_assertion = &affected_assertion.unwrap()[0];
        assert!(
            matches!( affected_assertion, Assertion::Affected {start_version,end_version}
                if start_version == "0.0.0-0"
                && end_version == "0.14.10"
            )
        );

        let fixed_assertions = loaded_advisory_vulnerability.fixed_assertions(()).await?;
        assert_eq!(1, fixed_assertions.assertions.len());

        let fixed_assertion = fixed_assertions.assertions.get("pkg://cargo/hyper");
        assert!(fixed_assertion.is_some());

        let fixed_assertion = fixed_assertion.unwrap();
        assert_eq!(1, fixed_assertion.len());

        let fixed_assertion = &fixed_assertion[0];
        assert!(matches!( fixed_assertion, Assertion::Fixed{version }
            if version == "0.14.10"
        ));

         */

        let advisory_vuln = loaded_advisory
            .get_vulnerability("CVE-2021-32714", ())
            .await?;
        assert!(advisory_vuln.is_some());

        let advisory_vuln = advisory_vuln.unwrap();
        let scores = advisory_vuln.cvss3_scores(()).await?;
        assert_eq!(1, scores.len());

        let score = scores[0];
        assert_eq!(
            score.to_string(),
            "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:H/A:H"
        );

        assert!(loaded_advisory
            .get_vulnerability("CVE-8675309", ())
            .await?
            .is_none());

        Ok(())
    }
}
