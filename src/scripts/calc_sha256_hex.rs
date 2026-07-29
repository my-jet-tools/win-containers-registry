use sha2::{Digest, Sha256};

/// The uploaded file is stored under the hash of its own content, so the same
/// build uploaded twice occupies one blob and a tag is just a pointer to a hash.
pub fn calc_sha256_hex(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);

    hasher
        .finalize()
        .iter()
        .map(|itm| format!("{:02x}", itm))
        .collect()
}
