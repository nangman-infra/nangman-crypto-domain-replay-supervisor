use std::collections::BTreeSet;

use crate::types::DomainRuntimeSpec;
use intel_candidate_app::error::AppResult;
use intel_candidate_app::storage::{ObjectStore, ObjectStoreConfig};

pub(crate) async fn estimate_input_keys(domain: &DomainRuntimeSpec) -> AppResult<usize> {
    let store = ObjectStore::connect(ObjectStoreConfig {
        bucket: domain.input_bucket.clone(),
        region: domain.input_region.clone(),
        profile: domain.input_profile.clone(),
        access_key_id: None,
        secret_access_key: None,
    })
    .await?;
    let suffixes = normalized_suffixes(&domain.input_suffixes);
    let mut unique = BTreeSet::new();
    for prefix in &domain.input_prefixes {
        for key in store.list_keys(prefix, domain.max_keys_per_prefix).await? {
            if suffixes.is_empty() || suffixes.iter().any(|suffix| key.ends_with(suffix)) {
                unique.insert(key);
            }
        }
    }
    Ok(unique.len())
}

fn normalized_suffixes(values: &[String]) -> BTreeSet<String> {
    values
        .iter()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect()
}
