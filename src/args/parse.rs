use super::s3::default_s3_output;
use super::validation::{
    absolute_path_arg, next_string, parse_non_negative_i64, validate_required_args,
};
use crate::keys::help_text;
use crate::types::Args;
use intel_candidate_app::error::{AppError, AppResult};
use std::collections::BTreeSet;
use std::path::PathBuf;

pub(crate) fn parse_args(values: impl Iterator<Item = String>) -> AppResult<Option<Args>> {
    let mut args = Args {
        manifest_file: PathBuf::new(),
        changed_triggers: BTreeSet::new(),
        output_dir: None,
        output_s3: None,
        now_ms: None,
    };
    let mut values = values.peekable();
    while let Some(arg) = values.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "--manifest-file" => {
                args.manifest_file =
                    absolute_path_arg(values.next(), "--manifest-file requires an absolute path")?;
            }
            "--changed-trigger" => {
                args.changed_triggers.insert(next_string(
                    &mut values,
                    "--changed-trigger requires a value",
                )?);
            }
            "--output-dir" => {
                args.output_dir = Some(absolute_path_arg(
                    values.next(),
                    "--output-dir requires an absolute path",
                )?);
            }
            "--output-s3-bucket" => {
                let bucket = next_string(&mut values, "--output-s3-bucket requires a bucket")?;
                let existing = args.output_s3.get_or_insert_with(default_s3_output);
                existing.bucket = bucket;
            }
            "--output-s3-region" => {
                let region = next_string(&mut values, "--output-s3-region requires a region")?;
                let existing = args.output_s3.get_or_insert_with(default_s3_output);
                existing.region = region;
            }
            "--output-s3-prefix" => {
                let prefix = next_string(&mut values, "--output-s3-prefix requires a prefix")?;
                let existing = args.output_s3.get_or_insert_with(default_s3_output);
                existing.prefix = prefix;
            }
            "--aws-profile" => {
                let profile = Some(next_string(
                    &mut values,
                    "--aws-profile requires a profile",
                )?);
                let existing = args.output_s3.get_or_insert_with(default_s3_output);
                existing.profile = profile;
            }
            "--now-ms" => {
                args.now_ms = Some(parse_non_negative_i64(&next_string(
                    &mut values,
                    "--now-ms requires a timestamp",
                )?)?);
            }
            other => {
                return Err(AppError::config(format!(
                    "unknown argument: {other}\n\n{}",
                    help_text()
                )));
            }
        }
    }
    validate_required_args(&args)?;
    Ok(Some(args))
}
