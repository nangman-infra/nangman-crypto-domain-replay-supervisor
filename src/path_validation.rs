use intel_candidate_app::error::{AppError, AppResult};
use std::path::{Component, Path};

pub(crate) fn validate_config_absolute_path(path: &Path, label: &str) -> AppResult<()> {
    validate_unambiguous_absolute_path(path, label).map_err(AppError::config)
}

pub(crate) fn validate_unambiguous_absolute_path(path: &Path, label: &str) -> Result<(), String> {
    if !path.is_absolute() {
        return Err(format!(
            "{label} must be an absolute path: {}",
            path_for_message(path)
        ));
    }

    let text = path.as_os_str().to_string_lossy();
    if text
        .split(['/', '\\'])
        .any(|segment| matches!(segment, "." | ".."))
        || path.components().any(|component| {
            matches!(
                component,
                Component::CurDir | Component::ParentDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "{label} must not contain relative path components: {}",
            path_for_message(path)
        ));
    }

    if text.chars().any(char::is_control) {
        return Err(format!(
            "{label} must not contain control characters: {}",
            path_for_message(path)
        ));
    }

    Ok(())
}

fn path_for_message(path: &Path) -> String {
    path.as_os_str()
        .to_string_lossy()
        .escape_debug()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::validate_unambiguous_absolute_path;
    use std::path::Path;

    #[test]
    fn rejects_relative_or_ambiguous_absolute_paths() {
        for path in [
            "relative-path",
            "/tmp/../domain-replay",
            "/tmp/./domain-replay",
            "/tmp/domain-replay\nout",
        ] {
            let error =
                validate_unambiguous_absolute_path(Path::new(path), "test path").unwrap_err();
            assert!(
                error.contains("test path"),
                "expected labelled error for {path:?}, got {error}"
            );
        }
    }

    #[test]
    fn accepts_plain_absolute_path() {
        validate_unambiguous_absolute_path(Path::new("/tmp/domain-replay"), "test path").unwrap();
    }
}
