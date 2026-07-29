use crate::models::ContainersError;

const MAX_NAME_LEN: usize = 128;

/// Container names and tags come from the url and become file-system paths,
/// so anything that could escape the storage folder is rejected up front.
pub fn validate_name(field: &'static str, value: &str) -> Result<(), ContainersError> {
    if value.is_empty() {
        return Err(ContainersError::InvalidName {
            field,
            reason: "must not be empty".to_string(),
        });
    }

    if value.len() > MAX_NAME_LEN {
        return Err(ContainersError::InvalidName {
            field,
            reason: format!("must not be longer than {} chars", MAX_NAME_LEN),
        });
    }

    for c in value.chars() {
        let is_allowed = c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-';

        if !is_allowed {
            return Err(ContainersError::InvalidName {
                field,
                reason: format!(
                    "'{}' is not allowed. Only a-z, A-Z, 0-9, '.', '_' and '-' are allowed",
                    c
                ),
            });
        }
    }

    if value.starts_with('.') || value.starts_with('-') {
        return Err(ContainersError::InvalidName {
            field,
            reason: "must not start with '.' or '-'".to_string(),
        });
    }

    Ok(())
}
