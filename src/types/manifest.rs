use super::constants::DEFAULT_AWS_REGION;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct DomainReplayManifest {
    pub(crate) schema_version: String,
    pub(crate) manifest_id: String,
    pub(crate) domains: Vec<DomainRuntimeSpec>,
    #[serde(default)]
    pub(crate) authority_migration_records: Vec<AuthorityMigrationInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct DomainRuntimeSpec {
    pub(crate) domain_id: String,
    pub(crate) target_app: String,
    pub(crate) target_mode: String,
    pub(crate) input_bucket: String,
    #[serde(default = "default_region")]
    pub(crate) input_region: String,
    #[serde(default)]
    pub(crate) input_profile: Option<String>,
    pub(crate) input_prefixes: Vec<String>,
    #[serde(default)]
    pub(crate) input_suffixes: Vec<String>,
    #[serde(default = "default_max_keys")]
    pub(crate) max_keys_per_prefix: usize,
    #[serde(default)]
    pub(crate) replay_triggers: Vec<String>,
    pub(crate) output_bucket: String,
    #[serde(default = "default_region")]
    pub(crate) output_region: String,
    pub(crate) output_report_prefix: String,
    pub(crate) command_template: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct AuthorityMigrationInput {
    pub(crate) authority_scope: String,
    #[serde(default)]
    pub(crate) from_app: Option<String>,
    pub(crate) to_app: String,
    pub(crate) migration_reason: String,
    #[serde(default)]
    pub(crate) effective_at_ms: Option<i64>,
}

fn default_region() -> String {
    DEFAULT_AWS_REGION.to_owned()
}

fn default_max_keys() -> usize {
    10_000
}
