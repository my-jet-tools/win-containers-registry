/// Container and tag addressed as a single `{container_name}:{tag}` string,
/// e.g. `mt4-bridge:0.1.0`.
pub struct ContainerId {
    pub container_name: String,
    pub tag: String,
}
