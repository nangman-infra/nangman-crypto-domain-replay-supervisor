use super::*;

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
    Ok(Some(args))
}

fn absolute_path_arg(value: Option<String>, message: &str) -> AppResult<PathBuf> {
    let value = value.ok_or_else(|| AppError::config(message))?;
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(AppError::config(format!(
            "{message}; got {}",
            path.display()
        )));
    }
    Ok(path)
}

fn next_string(
    values: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    message: &str,
) -> AppResult<String> {
    values.next().ok_or_else(|| AppError::config(message))
}

fn parse_non_negative_i64(value: &str) -> AppResult<i64> {
    let parsed = value
        .parse::<i64>()
        .map_err(|error| AppError::config(format!("invalid timestamp {value}: {error}")))?;
    if parsed < 0 {
        return Err(AppError::config("timestamp must be non-negative"));
    }
    Ok(parsed)
}

pub(crate) fn default_s3_output() -> S3OutputArgs {
    S3OutputArgs {
        bucket: String::new(),
        region: DEFAULT_AWS_REGION.to_owned(),
        prefix: String::new(),
        profile: None,
    }
}
