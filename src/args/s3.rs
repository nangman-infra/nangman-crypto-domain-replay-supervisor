use crate::types::{DEFAULT_AWS_REGION, S3OutputArgs};

pub(super) fn default_s3_output() -> S3OutputArgs {
    S3OutputArgs {
        bucket: String::new(),
        region: DEFAULT_AWS_REGION.to_owned(),
        prefix: String::new(),
        profile: None,
    }
}
