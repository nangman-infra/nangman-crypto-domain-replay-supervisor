use super::jsonl::{authority_migrations_jsonl, commands_jsonl};
use crate::keys::{
    authority_migration_records_key, checksum_json, control_plane_run_record_key,
    supervisor_commands_key, supervisor_report_key,
};
use crate::types::{DomainReplaySupervisorReport, S3OutputArgs};
use intel_candidate_app::error::AppResult;
use intel_candidate_app::storage::{ObjectStore, ObjectStoreConfig};

pub(crate) async fn write_s3_outputs(
    s3: &S3OutputArgs,
    report: &mut DomainReplaySupervisorReport,
    created_at_ms: i64,
    supervisor_run_id: &str,
    control_plane_run_record_id: &str,
) -> AppResult<Vec<String>> {
    let store = ObjectStore::connect(ObjectStoreConfig {
        bucket: s3.bucket.clone(),
        region: s3.region.clone(),
        profile: s3.profile.clone(),
        access_key_id: None,
        secret_access_key: None,
    })
    .await?;
    let report_key = supervisor_report_key(&s3.prefix, created_at_ms, supervisor_run_id);
    report.report_key = Some(report_key.clone());
    report.control_plane_run_record.output_report_key = Some(report_key.clone());
    report.control_plane_run_record.checksum = String::new();
    report.control_plane_run_record.checksum = checksum_json(&report.control_plane_run_record)?;
    report.checksum = String::new();
    report.checksum = checksum_json(report)?;
    store
        .put_bytes_idempotent(
            &report_key,
            serde_json::to_vec_pretty(report)?,
            "application/json",
        )
        .await?;

    let commands_key = supervisor_commands_key(&s3.prefix, created_at_ms, supervisor_run_id);
    store
        .put_bytes_idempotent(
            &commands_key,
            commands_jsonl(report)?.into_bytes(),
            "application/x-ndjson",
        )
        .await?;

    let run_record_key =
        control_plane_run_record_key(&s3.prefix, created_at_ms, control_plane_run_record_id);
    store
        .put_bytes_idempotent(
            &run_record_key,
            serde_json::to_vec_pretty(&report.control_plane_run_record)?,
            "application/json",
        )
        .await?;

    let mut output_s3_uris = vec![
        format!("s3://{}/{}", s3.bucket, report_key),
        format!("s3://{}/{}", s3.bucket, commands_key),
        format!("s3://{}/{}", s3.bucket, run_record_key),
    ];
    if !report.authority_migration_records.is_empty() {
        let migrations_key =
            authority_migration_records_key(&s3.prefix, created_at_ms, control_plane_run_record_id);
        store
            .put_bytes_idempotent(
                &migrations_key,
                authority_migrations_jsonl(&report.authority_migration_records)?.into_bytes(),
                "application/x-ndjson",
            )
            .await?;
        output_s3_uris.push(format!("s3://{}/{}", s3.bucket, migrations_key));
    }
    Ok(output_s3_uris)
}
