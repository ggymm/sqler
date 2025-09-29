use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "with-sqlx")]
use sqlx::AnyPool;

#[cfg(feature = "with-mongodb")]
use mongodb::Client as MongoClient;

#[cfg(feature = "with-redis")]
use redis::Client as RedisClient;

#[cfg(feature = "with-mssql")]
use tiberius::Client as MssqlClient;

#[cfg(feature = "with-oracle")]
use oracle::Connection as OracleConnection;

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum DbKind {
    Postgres = 1,
    Mysql = 2,
    Sqlite = 3,
    SqlServer = 4,
    Oracle = 5,
    MongoDb = 6,
    Redis = 7,
}

impl DbKind {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            1 => Some(DbKind::Postgres),
            2 => Some(DbKind::Mysql),
            3 => Some(DbKind::Sqlite),
            4 => Some(DbKind::SqlServer),
            5 => Some(DbKind::Oracle),
            6 => Some(DbKind::MongoDb),
            7 => Some(DbKind::Redis),
            _ => None,
        }
    }
}

pub enum Connection {
    #[cfg(feature = "with-sqlx")]
    Sqlx(AnyPool), // postgres/mysql/sqlite via any

    #[cfg(feature = "with-mongodb")]
    Mongo {
        client: MongoClient,
        db_name: String,
    },

    #[cfg(feature = "with-redis")]
    Redis(RedisClient),

    #[cfg(feature = "with-mssql")]
    Mssql, // Placeholder until implemented

    #[cfg(feature = "with-oracle")]
    Oracle(OracleConnection),

}

pub struct Context {
    pub conns: RwLock<HashMap<u64, Connection>>,
    pub next_id: AtomicU64,
    pub last_error: RwLock<Option<String>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            conns: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            last_error: RwLock::new(None),
        }
    }

    pub fn set_error(&self, msg: impl Into<String>) {
        *self.last_error.write() = Some(msg.into());
    }

    pub fn take_error(&self) -> Option<String> {
        self.last_error.write().take()
    }

    pub fn insert_conn(&self, conn: Connection) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.conns.write().insert(id, conn);
        id
    }

    pub fn remove_conn(&self, id: u64) -> Option<Connection> {
        self.conns.write().remove(&id)
    }

    pub fn get_conn(&self, id: u64) -> Option<ConnectionRef<'_>> {
        if self.conns.read().contains_key(&id) {
            Some(ConnectionRef { ctx: self, id })
        } else {
            None
        }
    }
}

pub struct ConnectionRef<'a> {
    ctx: &'a Context,
    id: u64,
}

impl<'a> ConnectionRef<'a> {
    pub fn with<R>(&self, f: impl FnOnce(&Connection) -> R) -> Option<R> {
        let map = self.ctx.conns.read();
        map.get(&self.id).map(f)
    }
}

pub static RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("sqler-rt")
        .build()
        .expect("create tokio runtime")
});
