use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Hex digits kept from the SHA-256 digest of a document identity. 32 hex
/// digits is 128 bits, so accidental collisions are negligible (birthday bound
/// around 2^64 documents) while the ID stays short enough to read in logs.
pub const DOCUMENT_ID_HEX_LEN: usize = 32;

/// Stable, store-independent document ID: `doc_<32 lowercase hex>`, the first
/// 128 bits of SHA-256 over an unambiguous encoding of
/// `(source_id, external_id, uri)`. Each field is written as an 8-byte
/// big-endian UTF-8 byte length followed by its bytes, so no two distinct
/// triples share an encoding. Unlike `std::hash::DefaultHasher`, this output is
/// fixed across Rust releases, and persisted IDs depend on it.
pub fn stable_document_id(source_id: &str, external_id: &str, uri: &str) -> String {
    let mut hasher = Sha256::new();
    for field in [source_id, external_id, uri] {
        hasher.update((field.len() as u64).to_be_bytes());
        hasher.update(field.as_bytes());
    }
    let digest = hex::encode(hasher.finalize());
    format!("doc_{}", &digest[..DOCUMENT_ID_HEX_LEN])
}

/// Normalized retrievable unit before chunking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub document_id: String,
    pub corpus_id: String,
    pub source_id: String,
    pub external_id: Option<String>,
    pub library_id: Option<String>,
    pub title: String,
    pub uri: Option<String>,
    pub version: Option<String>,
    pub mime_type: Option<String>,
    pub content_text: String,
    pub metadata_json: String,
    pub checksum: String,
    pub superseded_by_document_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Golden value computed independently (Python `hashlib`, same length-prefixed
    /// encoding). If this changes, every persisted document ID changes too.
    #[test]
    fn stable_document_id_golden_value() {
        assert_eq!(
            stable_document_id("src-a", "ext-1", "/path/doc.md"),
            "doc_cd2e53647dcfb96114e230663cd643f5"
        );
    }

    #[test]
    fn stable_document_id_encoding_is_unambiguous() {
        // Without length prefixes these two triples would concatenate identically.
        assert_ne!(
            stable_document_id("ab", "c", "u"),
            stable_document_id("a", "bc", "u")
        );
        assert_ne!(
            stable_document_id("s", "ab", ""),
            stable_document_id("s", "a", "b")
        );
    }
}
