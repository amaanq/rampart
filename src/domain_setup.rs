//! Expected DNS records and verification state for alias-domain onboarding.

use std::collections::BTreeMap;

use anyhow::{Context, Result};
use hickory_resolver::TokioResolver;
use hickory_resolver::config::{CLOUDFLARE, GOOGLE, NameServerConfig, ResolverConfig};
use hickory_resolver::net::runtime::TokioRuntimeProvider;
use hickory_resolver::proto::rr::RData;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

const SPF_VALUE: &str = "v=spf1 mx ~all";
const DMARC_VALUE: &str = "v=DMARC1; p=quarantine;";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct DkimRecord {
    pub algorithm: String,
    pub selector: String,
    pub value: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RecordStatus {
    #[default]
    Pending,
    Found,
    Mismatch,
}

impl RecordStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Found => "found",
            Self::Mismatch => "mismatch",
        }
    }

    const fn text(self) -> &'static str {
        match self {
            Self::Pending => "waiting",
            Self::Found => "found",
            Self::Mismatch => "mismatch",
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct DnsObservation {
    pub status: RecordStatus,
    #[serde(default)]
    pub expected: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed: Vec<String>,
}

pub(crate) type DnsStatus = BTreeMap<String, DnsObservation>;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SetupRecord {
    pub id: String,
    pub group: &'static str,
    pub kind: &'static str,
    pub host: String,
    pub fqdn: String,
    pub value: String,
    pub provider_value: String,
    pub value_label: &'static str,
    pub priority: Option<u16>,
    pub status: RecordStatus,
    pub status_label: &'static str,
    pub status_text: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SetupState {
    Setup,
    Ready,
    Attention,
}

impl SetupState {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Setup => "setup",
            Self::Ready => "ready",
            Self::Attention => "attention",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DomainSetup {
    pub domain_id: i64,
    pub domain: String,
    pub state: SetupState,
    pub state_label: &'static str,
    pub summary: String,
    pub dkim_pending: bool,
    pub records: Vec<SetupRecord>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub checked_at: Option<OffsetDateTime>,
}

impl DomainSetup {
    pub(crate) fn all_verified(&self) -> bool {
        !self.dkim_pending
            && self
                .records
                .iter()
                .all(|record| record.status == RecordStatus::Found)
    }
}

pub(crate) fn parse_dkim_records(value: &Value) -> Vec<DkimRecord> {
    serde_json::from_value(value.clone()).unwrap_or_default()
}

pub(crate) fn parse_dns_status(value: &Value) -> DnsStatus {
    serde_json::from_value(value.clone()).unwrap_or_default()
}

pub(crate) fn build(
    domain_id: i64,
    domain: &str,
    public_mx_hostname: &str,
    dkim_records: &[DkimRecord],
    status: &DnsStatus,
    checked_at: Option<OffsetDateTime>,
    verified_at: Option<OffsetDateTime>,
) -> DomainSetup {
    let mx = public_mx_hostname.trim_end_matches('.');
    let mut records = vec![
        record(
            "mx",
            "receive",
            "MX",
            "@",
            domain,
            format!("10 {mx}."),
            status,
        ),
        record(
            "spf",
            "reply",
            "TXT",
            "@",
            domain,
            SPF_VALUE.to_owned(),
            status,
        ),
        record(
            "dmarc",
            "reply",
            "TXT",
            "_dmarc",
            &format!("_dmarc.{domain}"),
            DMARC_VALUE.to_owned(),
            status,
        ),
    ];

    let mut has_rsa = false;
    let mut has_ed25519 = false;
    for dkim in dkim_records {
        has_rsa |= dkim.algorithm == "rsa";
        has_ed25519 |= dkim.algorithm == "ed25519";
        let host = format!("{}._domainkey", dkim.selector);
        records.push(record(
            &format!("dkim:{}:{}", dkim.algorithm, dkim.selector),
            "reply",
            "TXT",
            &host,
            &format!("{host}.{domain}"),
            dkim.value.clone(),
            status,
        ));
    }

    let dkim_pending = !has_rsa || !has_ed25519;
    let all_found = !dkim_pending
        && records
            .iter()
            .all(|record| record.status == RecordStatus::Found);
    let state = if all_found {
        SetupState::Ready
    } else if verified_at.is_some() {
        SetupState::Attention
    } else {
        SetupState::Setup
    };
    let mx_found = records
        .iter()
        .find(|record| record.id == "mx")
        .is_some_and(|record| record.status == RecordStatus::Found);
    let summary = match state {
        SetupState::Ready => "DNS is configured. You can create aliases now.".to_owned(),
        SetupState::Attention => {
            "DNS changed after setup. Restore the records marked below.".to_owned()
        }
        SetupState::Setup if dkim_pending => {
            "Generating signing records. This normally takes a few seconds.".to_owned()
        }
        SetupState::Setup if mx_found => {
            "Receiving works. Authenticated replies are still pending.".to_owned()
        }
        SetupState::Setup => "Add the records below. We’ll detect them automatically.".to_owned(),
    };

    DomainSetup {
        domain_id,
        domain: domain.to_owned(),
        state,
        state_label: state.as_str(),
        summary,
        dkim_pending,
        records,
        checked_at,
    }
}

fn record(
    id: &str,
    group: &'static str,
    kind: &'static str,
    host: &str,
    fqdn: &str,
    value: String,
    status: &DnsStatus,
) -> SetupRecord {
    let (provider_value, value_label, priority) = provider_fields(kind, &value);
    let record_status = status
        .get(id)
        .filter(|item| values_match(kind, &item.expected, &value))
        .map(|item| item.status)
        .unwrap_or_default();
    SetupRecord {
        id: id.to_owned(),
        group,
        kind,
        host: host.to_owned(),
        fqdn: fqdn.trim_end_matches('.').to_owned(),
        value,
        provider_value,
        value_label,
        priority,
        status: record_status,
        status_label: record_status.as_str(),
        status_text: record_status.text(),
    }
}

fn provider_fields(kind: &str, value: &str) -> (String, &'static str, Option<u16>) {
    if kind != "MX" {
        return (value.to_owned(), "value", None);
    }
    let (priority, mail_server) = value
        .split_once(' ')
        .expect("MX setup values include a priority");
    (
        mail_server.trim_end_matches('.').to_owned(),
        "mail server",
        Some(priority.parse().expect("MX setup priority is numeric")),
    )
}

pub(crate) async fn check(records: &[SetupRecord]) -> Result<DnsStatus> {
    let resolver = verification_resolver()?;
    let mut tasks = tokio::task::JoinSet::new();
    for record in records.iter().cloned() {
        let resolver = resolver.clone();
        tasks.spawn(async move {
            let result = check_one(&resolver, &record).await;
            (record.id, result)
        });
    }

    let mut status = DnsStatus::new();
    while let Some(result) = tasks.join_next().await {
        let (id, observation) = result.context("joining DNS lookup")?;
        status.insert(id, observation?);
    }
    Ok(status)
}

fn verification_resolver() -> Result<TokioResolver> {
    TokioResolver::builder_with_config(
        verification_resolver_config(),
        TokioRuntimeProvider::default(),
    )
    .build()
    .context("building DNS verification resolver")
}

fn verification_resolver_config() -> ResolverConfig {
    let mut config = ResolverConfig::default();
    for ip in [CLOUDFLARE.ips[0], GOOGLE.ips[0]] {
        let mut name_server = NameServerConfig::udp_and_tcp(ip);
        name_server.trust_negative_responses = false;
        config.add_name_server(name_server);
    }
    config
}

async fn check_one(resolver: &TokioResolver, record: &SetupRecord) -> Result<DnsObservation> {
    let fqdn = absolute_dns_name(&record.fqdn);
    let observed = match record.kind {
        "MX" => match resolver.mx_lookup(fqdn.as_str()).await {
            Ok(lookup) => lookup
                .iter()
                .map(|mx| format!("{} {}", mx.preference(), mx.exchange()))
                .collect(),
            Err(error) if error.is_no_records_found() => Vec::new(),
            Err(error) => return Err(error).context(format!("looking up MX for {}", record.fqdn)),
        },
        "TXT" => match resolver.txt_lookup(fqdn.as_str()).await {
            Ok(lookup) => lookup
                .iter()
                .map(|txt| {
                    let bytes = txt
                        .txt_data()
                        .iter()
                        .flat_map(|chunk| chunk.iter().copied())
                        .collect::<Vec<_>>();
                    String::from_utf8_lossy(&bytes).into_owned()
                })
                .collect(),
            Err(error) if error.is_no_records_found() => Vec::new(),
            Err(error) => {
                return Err(error).context(format!("looking up TXT for {}", record.fqdn));
            }
        },
        kind => anyhow::bail!("unsupported DNS record type {kind}"),
    };
    let found = observed
        .iter()
        .any(|value| values_match(record.kind, value, &record.value));
    let status = if found {
        RecordStatus::Found
    } else if observed.is_empty() {
        RecordStatus::Pending
    } else {
        RecordStatus::Mismatch
    };
    Ok(DnsObservation {
        status,
        expected: record.value.clone(),
        observed,
    })
}

fn absolute_dns_name(name: &str) -> String {
    format!("{}.", name.trim_end_matches('.'))
}

fn values_match(kind: &str, observed: &str, expected: &str) -> bool {
    match kind {
        "MX" => {
            observed.trim().trim_end_matches('.').to_ascii_lowercase()
                == expected.trim().trim_end_matches('.').to_ascii_lowercase()
        }
        _ => observed.trim() == expected.trim(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dkim(algorithm: &str) -> DkimRecord {
        DkimRecord {
            algorithm: algorithm.to_owned(),
            selector: format!("v1-{algorithm}-20260718"),
            value: format!("v=DKIM1; k={algorithm}; h=sha256; p=key"),
        }
    }

    #[test]
    fn requires_both_dkim_algorithms_before_ready() {
        let records = vec![dkim("rsa"), dkim("ed25519")];
        let initial = build(
            1,
            "example.test",
            "mx.test",
            &records,
            &DnsStatus::new(),
            None,
            None,
        );
        assert_eq!(initial.records.len(), 5);
        assert!(!initial.dkim_pending);

        let status = initial
            .records
            .iter()
            .map(|record| {
                (
                    record.id.clone(),
                    DnsObservation {
                        status: RecordStatus::Found,
                        expected: record.value.clone(),
                        observed: vec![],
                    },
                )
            })
            .collect();
        let ready = build(1, "example.test", "mx.test", &records, &status, None, None);
        assert_eq!(ready.state, SetupState::Ready);
    }

    #[test]
    fn previously_verified_drift_needs_attention() {
        let setup = build(
            1,
            "example.test",
            "mx.test",
            &[dkim("rsa"), dkim("ed25519")],
            &DnsStatus::new(),
            Some(OffsetDateTime::UNIX_EPOCH),
            Some(OffsetDateTime::UNIX_EPOCH),
        );
        assert_eq!(setup.state, SetupState::Attention);
    }

    #[test]
    fn txt_values_remain_case_sensitive_for_dkim_keys() {
        assert!(values_match("MX", "10 MX.TEST.", "10 mx.test."));
        assert!(!values_match("TXT", "p=AbCd", "p=abcd"));
    }

    #[test]
    fn dns_queries_skip_system_search_domains() {
        assert_eq!(absolute_dns_name("example.test"), "example.test.");
        assert_eq!(absolute_dns_name("example.test."), "example.test.");
    }

    #[test]
    fn mx_provider_fields_are_separate_and_dotless() {
        assert_eq!(
            provider_fields("MX", "10 mx.rampart.email."),
            ("mx.rampart.email".to_owned(), "mail server", Some(10))
        );
    }
}
