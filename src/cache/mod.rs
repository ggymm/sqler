use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ConnectionsCache {
    #[serde(default)]
    pub next_id: usize,
    #[serde(default)]
    pub selected: Option<usize>,
    #[serde(default)]
    pub connections: Vec<ConnectionRecord>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ConnectionRecord {
    pub id: usize,
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub config: Option<StoredConnectionConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StoredConnectionConfig {
    Relational {
        host: String,
        port: u16,
        database: String,
        username: String,
        #[serde(default)]
        password: Option<String>,
    },
    Sqlite {
        file_path: String,
    },
    Mongo {
        connection_string: String,
    },
    Redis {
        host: String,
        port: u16,
        #[serde(default)]
        password: Option<String>,
    },
}

pub fn load_connections() -> Option<ConnectionsCache> {
    let path = cache_file_path();
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_connections(cache: &ConnectionsCache) -> io::Result<()> {
    let path = cache_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(cache).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    fs::write(path, data)
}

fn cache_file_path() -> PathBuf {
    cache_directory().join("connections.json")
}

fn cache_directory() -> PathBuf {
    if let Some(custom) = env::var_os("SQLER_CACHE_DIR") {
        return PathBuf::from(custom);
    }

    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home).join(".sqler");
    }

    if let Some(profile) = env::var_os("USERPROFILE") {
        return PathBuf::from(profile).join(".sqler");
    }

    env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(".sqler")
}
