use std::time::{SystemTime, UNIX_EPOCH};

use uuid;

use errors::*;
use super::{Resource, texture, bytes};

#[derive(Debug, Serialize, Deserialize)]
pub enum ResourceConcreteMetadata {
    Bytes(bytes::BytesMetadata),
    Texture(texture::TextureMetadata),
}

/// The descriptions of a resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceMetadata {
    time_created: u64,
    uuid: uuid::Uuid,
    metadata: ResourceConcreteMetadata,
}

impl ResourceMetadata {
    pub fn new(metadata: ResourceConcreteMetadata) -> ResourceMetadata {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        ResourceMetadata {
            time_created: timestamp,
            uuid: uuid::Uuid::new_v4(),
            metadata: metadata,
        }
    }

    pub fn new_as(tt: Resource) -> ResourceMetadata {
        let concrete = match tt {
            Resource::Bytes => ResourceConcreteMetadata::Bytes(bytes::BytesMetadata::new()),
            Resource::Texture => ResourceConcreteMetadata::Texture(texture::TextureMetadata::new()),
        };

        ResourceMetadata::new(concrete)
    }

    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }

    pub fn is(&self, tt: Resource) -> bool {
        self.file_type() == tt
    }

    pub fn file_type(&self) -> Resource {
        match &self.metadata {
            &ResourceConcreteMetadata::Bytes(_) => Resource::Bytes,
            &ResourceConcreteMetadata::Texture(_) => Resource::Texture,
        }
    }

    pub fn validate(&self, bytes: &[u8]) -> Result<()> {
        match &self.metadata {
            &ResourceConcreteMetadata::Bytes(ref metadata) => metadata.validate(&bytes),
            &ResourceConcreteMetadata::Texture(ref metadata) => metadata.validate(&bytes),
        }
    }

    pub fn build(&self, bytes: &[u8], mut out: &mut Vec<u8>) -> Result<()> {
        match &self.metadata {
            &ResourceConcreteMetadata::Texture(ref metadata) => metadata.build(&bytes, &mut out),
            &ResourceConcreteMetadata::Bytes(ref metadata) => metadata.build(&bytes, &mut out),
        }
    }
}