mod jsonl;
mod local;
mod s3;

#[cfg(test)]
mod tests;

#[cfg(test)]
use local::local_output_path;
pub(crate) use local::write_local_outputs;
pub(crate) use s3::write_s3_outputs;
