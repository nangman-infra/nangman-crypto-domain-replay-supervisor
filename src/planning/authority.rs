use crate::types::{
    AUTHORITY_MIGRATION_RECORD_SCHEMA_VERSION, AuthorityMigrationInput, AuthorityMigrationRecord,
};
use intel_candidate_app::hash::stable_id;

pub(crate) fn build_authority_migration_records(
    inputs: &[AuthorityMigrationInput],
    control_plane_run_record_id: &str,
    created_at_ms: i64,
    manifest_id: &str,
    manifest_checksum: &str,
) -> Vec<AuthorityMigrationRecord> {
    inputs
        .iter()
        .map(|input| {
            let effective_at_ms = input.effective_at_ms.unwrap_or(created_at_ms);
            let authority_migration_record_id = stable_id(
                "authority_migration",
                &[
                    control_plane_run_record_id,
                    &input.authority_scope,
                    input.from_app.as_deref().unwrap_or("none"),
                    &input.to_app,
                    &effective_at_ms.to_string(),
                ],
            );
            AuthorityMigrationRecord {
                schema_version: AUTHORITY_MIGRATION_RECORD_SCHEMA_VERSION.to_owned(),
                authority_migration_record_id: authority_migration_record_id.clone(),
                control_plane_run_record_id: control_plane_run_record_id.to_owned(),
                authority_scope: input.authority_scope.clone(),
                from_app: input.from_app.clone(),
                to_app: input.to_app.clone(),
                migration_reason: input.migration_reason.clone(),
                effective_at_ms,
                created_at_ms,
                manifest_id: manifest_id.to_owned(),
                manifest_checksum: manifest_checksum.to_owned(),
                idempotency_key: stable_id(
                    "authority_migration_idem",
                    &[&authority_migration_record_id],
                ),
            }
        })
        .collect()
}
