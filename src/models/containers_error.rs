#[derive(Debug)]
pub enum ContainersError {
    Unauthorized,
    TagIsMissing(String),
    InvalidName { field: &'static str, reason: String },
    EmptyContent,
    NotZipArchive,
    ContainerNotFound(String),
    TagNotFound { container: String, tag: String },
}
