use crate::path_validation::validate_config_absolute_path;
use crate::types::Args;
use intel_candidate_app::error::{AppError, AppResult};
use std::path::PathBuf;

pub(super) fn absolute_path_arg(value: Option<String>, message: &str) -> AppResult<PathBuf> {
    let value = value.ok_or_else(|| AppError::config(message))?;
    let path = PathBuf::from(value);
    let label = message.split(" requires").next().unwrap_or("path");
    validate_config_absolute_path(&path, label)
        .map_err(|error| AppError::config(format!("{message}; {error}")))?;
    Ok(path)
}

pub(super) fn next_string(
    values: &mut impl Iterator<Item = String>,
    message: &str,
) -> AppResult<String> {
    values.next().ok_or_else(|| AppError::config(message))
}

pub(super) fn parse_non_negative_i64(value: &str) -> AppResult<i64> {
    let parsed = value
        .parse::<i64>()
        .map_err(|error| AppError::config(format!("invalid timestamp {value}: {error}")))?;
    if parsed < 0 {
        return Err(AppError::config("timestamp must be non-negative"));
    }
    Ok(parsed)
}

pub(super) fn validate_required_args(args: &Args) -> AppResult<()> {
    if args.manifest_file.as_os_str().is_empty() {
        return Err(AppError::config("--manifest-file is required"));
    }
    if args.changed_triggers.is_empty() {
        return Err(AppError::config("--changed-trigger is required"));
    }
    if let Some(s3) = args.output_s3.as_ref()
        && (s3.bucket.trim().is_empty() || s3.prefix.trim().is_empty())
    {
        return Err(AppError::config(
            "--output-s3-bucket and --output-s3-prefix are required for S3 output",
        ));
    }
    Ok(())
}
