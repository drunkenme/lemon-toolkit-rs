use std::path::Path;
use errors::*;
use workspace::Database;
use super::ResourceUnderlyingMetadata;

#[derive(Debug, Serialize, Deserialize)]
pub struct BytesMetadata;

impl BytesMetadata {
    pub fn new() -> Self {
        BytesMetadata {}
    }
}

impl ResourceUnderlyingMetadata for BytesMetadata {
    fn validate(&self, _: &[u8]) -> Result<()> {
        Ok(())
    }

    fn build(&self, _: &Database, _: &Path, bytes: &[u8], mut out: &mut Vec<u8>) -> Result<()> {
        out.resize(bytes.len(), 0);
        out.copy_from_slice(&bytes);
        Ok(())
    }
}