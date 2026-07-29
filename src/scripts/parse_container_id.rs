use crate::models::{ContainerId, ContainersError};

/// Parses `{container_name}:{tag}` — e.g. `mt4-bridge:0.1.0`.
///
/// The split is on the **last** `:`, so a container name is free to contain
/// colons of its own; only what follows the final one is the tag.
pub fn parse_container_id(src: &str) -> Result<ContainerId, ContainersError> {
    let Some(index) = src.rfind(':') else {
        return Err(ContainersError::TagIsMissing(src.to_string()));
    };

    let container_name = &src[..index];
    let tag = &src[index + 1..];

    crate::scripts::validate_name("containerName", container_name)?;
    crate::scripts::validate_name("tag", tag)?;

    Ok(ContainerId {
        container_name: container_name.to_string(),
        tag: tag.to_string(),
    })
}
