use my_http_server::*;

use crate::models::ContainersError;

impl From<ContainersError> for HttpFailResult {
    fn from(src: ContainersError) -> Self {
        match src {
            ContainersError::Unauthorized => {
                HttpFailResult::as_unauthorized(Some("Invalid or missing api key"))
            }
            ContainersError::TagIsMissing(container) => {
                HttpFailResult::as_validation_error(format!(
                    "'{}' has no tag. Expected format is {{container_name}}:{{tag}}, e.g. mt4-bridge:0.1.0",
                    container
                ))
            }
            ContainersError::InvalidName { field, reason } => {
                HttpFailResult::as_validation_error(format!("Invalid {}: {}", field, reason))
            }
            ContainersError::EmptyContent => HttpFailResult::as_validation_error("Body is empty"),
            ContainersError::NotZipArchive => {
                HttpFailResult::as_validation_error("Body is not a zip archive")
            }
            ContainersError::ContainerNotFound(container) => {
                HttpFailResult::as_not_found(format!("Container '{}' not found", container), false)
            }
            ContainersError::TagNotFound { container, tag } => HttpFailResult::as_not_found(
                format!("Tag '{}' not found in container '{}'", tag, container),
                false,
            ),
        }
    }
}
