use crate::types::{DomainReplayManifest, MANIFEST_SCHEMA_VERSION};
use intel_candidate_app::error::{AppError, AppResult};

pub(crate) fn validate_manifest(manifest: &DomainReplayManifest) -> AppResult<()> {
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(AppError::config(format!(
            "manifest schema_version must be {MANIFEST_SCHEMA_VERSION}"
        )));
    }
    if manifest.manifest_id.trim().is_empty() {
        return Err(AppError::config("manifest_id is required"));
    }
    if manifest.domains.is_empty() {
        return Err(AppError::config("at least one domain is required"));
    }
    for domain in &manifest.domains {
        if domain.domain_id.trim().is_empty()
            || domain.target_app.trim().is_empty()
            || domain.target_mode.trim().is_empty()
            || domain.input_bucket.trim().is_empty()
            || domain.input_prefixes.is_empty()
            || domain.output_bucket.trim().is_empty()
            || domain.output_report_prefix.trim().is_empty()
            || domain.command_template.is_empty()
        {
            return Err(AppError::config(format!(
                "domain {} is missing required fields",
                domain.domain_id
            )));
        }
    }
    for record in &manifest.authority_migration_records {
        if record.authority_scope.trim().is_empty()
            || record.to_app.trim().is_empty()
            || record.migration_reason.trim().is_empty()
        {
            return Err(AppError::config(
                "authority_migration_records require authority_scope, to_app, and migration_reason",
            ));
        }
        if record
            .effective_at_ms
            .is_some_and(|effective_at_ms| effective_at_ms < 0)
        {
            return Err(AppError::config(
                "authority_migration_records.effective_at_ms must be non-negative",
            ));
        }
    }
    Ok(())
}
