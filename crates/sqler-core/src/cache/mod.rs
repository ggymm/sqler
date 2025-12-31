use std::{
    fs, io,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, KeyInit},
};
use thiserror::Error;

use crate::{DataSource, SavedQuery, TableInfo, cache_queries, cache_tables, sources_db};

pub type ArcCache = Arc<RwLock<AppCache>>;

const ENCRYPTION_KEY: [u8; 32] = [
    0x7f, 0x3e, 0x9a, 0x5c, 0x2b, 0x8f, 0x1d, 0x6e, 0x4a, 0x0c, 0x7b, 0x9f, 0x3d, 0x5a, 0x8e, 0x2c, 0x1f, 0x6b, 0x4d,
    0x9a, 0x0e, 0x7c, 0x3f, 0x5b, 0x8d, 0x2a, 0x9e, 0x1c, 0x6f, 0x4b, 0x0d, 0x7a,
];

const NONCE: [u8; 12] = [0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0x07, 0x18, 0x29, 0x3a, 0x4b, 0x5c];

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),
}

pub struct AppCache {
    sources: Vec<DataSource>,
    sources_path: PathBuf,
}

impl AppCache {
    pub fn init() -> Result<ArcCache, CacheError> {
        let sources_path = sources_db();

        let sources = if sources_path.exists() {
            let encrypted = fs::read(&sources_path)?;
            let decrypted = Self::decrypt(&encrypted)?;
            serde_json::from_slice(&decrypted)?
        } else {
            vec![]
        };

        Ok(Arc::new(RwLock::new(Self { sources, sources_path })))
    }

    pub fn sources(&self) -> &[DataSource] {
        &self.sources
    }

    pub fn sources_mut(&mut self) -> &mut Vec<DataSource> {
        &mut self.sources
    }

    pub fn sources_update(&mut self) -> Result<(), CacheError> {
        let json = serde_json::to_vec(&self.sources)?;
        let encrypted = Self::encrypt(&json)?;
        fs::write(&self.sources_path, encrypted)?;
        Ok(())
    }

    pub fn tables(
        &self,
        uuid: &str,
    ) -> Result<Vec<TableInfo>, CacheError> {
        let path = cache_tables(uuid);
        if !path.exists() {
            return Ok(vec![]);
        }

        let data = fs::read(&path)?;
        let tables = serde_json::from_slice(&data)?;
        Ok(tables)
    }

    pub fn tables_update(
        &self,
        uuid: &str,
        tables: &[TableInfo],
    ) -> Result<(), CacheError> {
        let path = cache_tables(uuid);
        let json = serde_json::to_vec(tables)?;
        fs::write(&path, json)?;
        Ok(())
    }

    pub fn queries(
        &self,
        uuid: &str,
    ) -> Result<Vec<SavedQuery>, CacheError> {
        let path = cache_queries(uuid);
        if !path.exists() {
            return Ok(vec![]);
        }

        let data = fs::read(&path)?;
        let queries = serde_json::from_slice(&data)?;
        Ok(queries)
    }

    pub fn queries_update(
        &self,
        uuid: &str,
        queries: &[SavedQuery],
    ) -> Result<(), CacheError> {
        let path = cache_queries(uuid);
        let json = serde_json::to_vec(queries)?;
        fs::write(&path, json)?;
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
            .map_err(|e| CacheError::Decryption(e.to_string()))
    }
}

#[cfg(test)]
mod tests {}
