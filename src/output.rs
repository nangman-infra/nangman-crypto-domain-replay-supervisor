use crate::keys::{
    authority_migration_records_key, checksum_json, control_plane_run_record_key,
    supervisor_commands_key, supervisor_report_key,
};
use crate::types::{AuthorityMigrationRecord, DomainReplaySupervisorReport, S3OutputArgs};
use intel_candidate_app::error::{AppError, AppResult};
use intel_candidate_app::storage::{ObjectStore, ObjectStoreConfig};
use std::fs;
use std::path::Path;

pub(crate) fn write_local_outputs(
    report: &DomainReplaySupervisorReport,
    output_dir: &Path,
    supervisor_run_id: &str,
    control_plane_run_record_id: &str,
) -> AppResult<Vec<String>> {
    fs::create_dir_all(output_dir).map_err(|error| {
        AppError::config(format!(
            "create output dir {}: {error}",
            output_dir.display()
        ))
    })?;
    let report_path = output_dir.join(format!("{supervisor_run_id}.report.json"));
    let commands_path = output_dir.join(format!("{supervisor_run_id}.commands.jsonl"));
    fs::write(&report_path, serde_json::to_vec_pretty(report)?).map_err(|error| {
        AppError::config(format!("write report {}: {error}", report_path.display()))
    })?;
    fs::write(&commands_path, commands_jsonl(report)?).map_err(|error| {
        AppError::config(format!(
            "write commands {}: {error}",
            commands_path.display()
        ))
    })?;
    let run_record_path = output_dir.join(format!(
        "{control_plane_run_record_id}.control-plane-run-record.json"
    ));
    fs::write(
        &run_record_path,
        serde_json::to_vec_pretty(&report.control_plane_run_record)?,
    )
    .map_err(|error| {
        AppError::config(format!(
            "write control-plane run record {}: {error}",
            run_record_path.display()
        ))
    })?;

    let mut output_files = vec![
        report_path.display().to_string(),
        commands_path.display().to_string(),
        run_record_path.display().to_string(),
    ];
    if !report.authority_migration_records.is_empty() {
        let migrations_path = output_dir.join(format!(
            "{control_plane_run_record_id}.authority-migrations.jsonl"
        ));
        fs::write(
            &migrations_path,
            authority_migrations_jsonl(&report.authority_migration_records)?,
        )
        .map_err(|error| {
            AppError::config(format!(
                "write authority migrations {}: {error}",
                migrations_path.display()
            ))
        })?;
        output_files.push(migrations_path.display().to_string());
    }
    Ok(output_files)
}

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

fn commands_jsonl(report: &DomainReplaySupervisorReport) -> AppResult<String> {
    let commands = report
        .workflow_commands
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    Ok(format!("{commands}\n"))
}

fn authority_migrations_jsonl(records: &[AuthorityMigrationRecord]) -> AppResult<String> {
    let migrations = records
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    Ok(format!("{migrations}\n"))
}
