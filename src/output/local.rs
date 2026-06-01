use super::jsonl::{authority_migrations_jsonl, commands_jsonl};
use crate::path_validation::validate_config_absolute_path;
use crate::types::DomainReplaySupervisorReport;
use intel_candidate_app::error::{AppError, AppResult};
use intel_candidate_app::time::path_segment;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) fn write_local_outputs(
    report: &DomainReplaySupervisorReport,
    output_dir: &Path,
    supervisor_run_id: &str,
    control_plane_run_record_id: &str,
) -> AppResult<Vec<String>> {
    validate_config_absolute_path(output_dir, "output dir")?;
    fs::create_dir_all(output_dir).map_err(|error| {
        AppError::config(format!(
            "create output dir {}: {error}",
            output_dir.display()
        ))
    })?;
    let report_path = local_output_path(output_dir, supervisor_run_id, "report.json")?;
    let commands_path = local_output_path(output_dir, supervisor_run_id, "commands.jsonl")?;
    write_output_file(&report_path, &serde_json::to_vec_pretty(report)?, "report")?;
    write_output_file(
        &commands_path,
        commands_jsonl(report)?.as_bytes(),
        "commands",
    )?;
    let run_record_path = local_output_path(
        output_dir,
        control_plane_run_record_id,
        "control-plane-run-record.json",
    )?;
    write_output_file(
        &run_record_path,
        &serde_json::to_vec_pretty(&report.control_plane_run_record)?,
        "control-plane run record",
    )?;

    let mut output_files = vec![
        report_path.display().to_string(),
        commands_path.display().to_string(),
        run_record_path.display().to_string(),
    ];
    if !report.authority_migration_records.is_empty() {
        let migrations_path = local_output_path(
            output_dir,
            control_plane_run_record_id,
            "authority-migrations.jsonl",
        )?;
        write_output_file(
            &migrations_path,
            authority_migrations_jsonl(&report.authority_migration_records)?.as_bytes(),
            "authority migrations",
        )?;
        output_files.push(migrations_path.display().to_string());
    }
    Ok(output_files)
}

pub(super) fn local_output_path(output_dir: &Path, id: &str, suffix: &str) -> AppResult<PathBuf> {
    validate_file_id(id)?;
    Ok(output_dir.join(format!("{id}.{suffix}")))
}

fn validate_file_id(id: &str) -> AppResult<()> {
    if id.is_empty() {
        return Err(AppError::config("local output id must not be empty"));
    }
    if id == "." || id == ".." || id != path_segment(id) {
        return Err(AppError::config(
            "local output id must be a single safe path segment",
        ));
    }
    Ok(())
}

fn write_output_file(path: &Path, bytes: &[u8], label: &str) -> AppResult<()> {
    let mut file = create_output_file(path)?;
    file.write_all(bytes)
        .map_err(|error| AppError::config(format!("write {label} {}: {error}", path.display())))
}

fn create_output_file(path: &Path) -> AppResult<File> {
    let parent = path.parent().ok_or_else(|| {
        AppError::config(format!("output path has no parent: {}", path.display()))
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        AppError::config(format!("create output dir {}: {error}", parent.display()))
    })?;
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(AppError::config(format!(
            "output path must not be a symlink: {}",
            path.display()
        )));
    }
    File::create(path).map_err(|error| {
        AppError::config(format!("create output file {}: {error}", path.display()))
    })
}
