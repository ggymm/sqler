use std::{fs, io, path::PathBuf};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm,
};
use thiserror::Error;

use crate::driver::DataSource;

const ENCRYPTION_KEY: [u8; 32] = [
    0x7f, 0x3e, 0x9a, 0x5c, 0x2b, 0x8f, 0x1d, 0x6e, 0x4a, 0x0c, 0x7b, 0x9f, 0x3d, 0x5a, 0x8e, 0x2c, 0x1f, 0x6b, 0x4d,
    0x9a, 0x0e, 0x7c, 0x3f, 0x5b, 0x8d, 0x2a, 0x9e, 0x1c, 0x6f, 0x4b, 0x0d, 0x7a,
];

const NONCE: [u8; 12] = [0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0x07, 0x18, 0x29, 0x3a, 0x4b, 0x5c];

const CACHE_DIR: &str = ".sqler";
const CACHE_FILE: &str = "sources.enc";

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Cache directory not found")]
    DirectoryNotFound,
}

pub struct CacheApp {
    sources: Vec<DataSource>,
    sources_path: PathBuf,
}

impl CacheApp {
    pub fn init() -> Result<Self, CacheError> {
        let cache_dir = dirs::home_dir()
            .map(|home| home.join(CACHE_DIR))
            .ok_or(CacheError::DirectoryNotFound)?;
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }

        let sources_path = cache_dir.join(CACHE_FILE);
        let sources = if sources_path.exists() {
            let encrypted = fs::read(&sources_path)?;
            let decrypted = Self::decrypt(&encrypted)?;
            let data: Vec<DataSource> = serde_json::from_slice(&decrypted)?;
            data
        } else {
            Vec::new()
        };

        Ok(Self { sources, sources_path })
    }

    pub fn sources(&self) -> &[DataSource] {
        &self.sources
    }

    pub fn sources_mut(&mut self) -> &mut Vec<DataSource> {
        &mut self.sources
    }

    pub fn sources_update(&mut self) -> Result<(), CacheError> {
        let data = self.sources.clone();
        let json = serde_json::to_vec(&data)?;
        let encrypted = Self::encrypt(&json)?;
        fs::write(&self.sources_path, encrypted)?;
        Ok(())
    }

    fn encrypt(data: &[u8]) -> Result<Vec<u8>, CacheError> {
        Aes256Gcm::new(&ENCRYPTION_KEY.into())
            .encrypt(&NONCE.into(), data)
            .map_err(|e| CacheError::Encryption(e.to_string()))
    }

    fn decrypt(data: &[u8]) -> Result<Vec<u8>, CacheError> {
        Aes256Gcm::new(&ENCRYPTION_KEY.into())
            .decrypt(&NONCE.into(), data)
            .map_err(|e| CacheError::Encryption(e.to_string()))
    }
}

#[cfg(test)]
mod tests {}
