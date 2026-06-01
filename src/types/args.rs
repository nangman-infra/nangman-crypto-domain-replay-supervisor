use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Args {
    pub(crate) manifest_file: PathBuf,
    pub(crate) changed_triggers: BTreeSet<String>,
    pub(crate) output_dir: Option<PathBuf>,
    pub(crate) output_s3: Option<S3OutputArgs>,
    pub(crate) now_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct S3OutputArgs {
    pub(crate) bucket: String,
    pub(crate) region: String,
    pub(crate) prefix: String,
    pub(crate) profile: Option<String>,
}
