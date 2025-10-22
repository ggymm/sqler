use super::{
    DatabaseDriver, DatabaseSession, DeleteReq, DriverError, InsertReq, QueryReq, QueryResp, UpdateReq, WriteResp,
};
use crate::option::{MongoDBHost, MongoDBOptions};

use mongodb::bson::{doc, to_document, Document};
use mongodb::sync::Client;

#[derive(Debug, Clone, Copy)]
pub struct MongoDBDriver;

struct MongoConnection {
    client: Client,
    default_db: String,
}

impl MongoConnection {
    fn new(
        client: Client,
        default_db: String,
    ) -> Self {
        Self { client, default_db }
    }

    fn resolve_collection<'a>(
        &'a self,
        full_name: &'a str,
    ) -> Result<(&'a str, &'a str), DriverError> {
        if let Some((db, coll)) = full_name.split_once('.') {
            if db.is_empty() || coll.is_empty() {
                return Err(DriverError::InvalidField("collection".into()));
            }
            Ok((db, coll))
        } else {
            if self.default_db.is_empty() {
                return Err(DriverError::InvalidField(
                    "collection 需要包含数据库前缀，例如 db.collection".into(),
                ));
            }
            Ok((self.default_db.as_str(), full_name))
        }
    }
}

impl DatabaseSession for MongoConnection {
    fn query(
        &mut self,
        request: QueryReq,
    ) -> Result<QueryResp, DriverError> {
        match request {
            QueryReq::Document { collection, filter } => {
                let (db, coll) = self.resolve_collection(&collection)?;
                let filter_doc =
                    to_document(&filter).map_err(|err| DriverError::Other(format!("解析过滤条件失败: {}", err)))?;

                let cursor = self
                    .client
                    .database(db)
                    .collection::<Document>(coll)
                    .find(filter_doc)
                    .run()
                    .map_err(|err| DriverError::Other(format!("执行查询失败: {}", err)))?;

                let mut docs = Vec::new();
                for doc in cursor {
                    let doc = doc.map_err(|err| DriverError::Other(format!("读取结果失败: {}", err)))?;
                    docs.push(
                        serde_json::to_value(&doc)
                            .map_err(|err| DriverError::Other(format!("序列化结果失败: {}", err)))?,
                    );
                }

                Ok(QueryResp::Documents(docs))
            }
            other => Err(DriverError::InvalidField(format!(
                "MongoDB 查询仅支持文档操作，收到: {:?}",
                other
            ))),
        }
    }

    fn insert(
        &mut self,
        request: InsertReq,
    ) -> Result<WriteResp, DriverError> {
        match request {
            InsertReq::Document { collection, document } => {
                let (db, coll) = self.resolve_collection(&collection)?;
                let doc = to_document(&document).map_err(|err| DriverError::Other(format!("解析文档失败: {}", err)))?;
                self.client
                    .database(db)
                    .collection::<Document>(coll)
                    .insert_one(doc)
                    .run()
                    .map_err(|err| DriverError::Other(format!("插入失败: {}", err)))?;
                Ok(WriteResp { affected: 1 })
            }
            other => Err(DriverError::InvalidField(format!(
                "MongoDB 插入仅支持文档操作，收到: {:?}",
                other
            ))),
        }
    }

    fn update(
        &mut self,
        request: UpdateReq,
    ) -> Result<WriteResp, DriverError> {
        match request {
            UpdateReq::Document {
                collection,
                filter,
                update,
            } => {
                let (db, coll) = self.resolve_collection(&collection)?;
                let filter_doc =
                    to_document(&filter).map_err(|err| DriverError::Other(format!("解析过滤条件失败: {}", err)))?;
                let update_doc =
                    to_document(&update).map_err(|err| DriverError::Other(format!("解析更新内容失败: {}", err)))?;

                let result = self
                    .client
                    .database(db)
                    .collection::<Document>(coll)
                    .update_many(filter_doc, update_doc)
                    .run()
                    .map_err(|err| DriverError::Other(format!("更新失败: {}", err)))?;
                Ok(WriteResp {
                    affected: result.modified_count,
                })
            }
            other => Err(DriverError::InvalidField(format!(
                "MongoDB 更新仅支持文档操作，收到: {:?}",
                other
            ))),
        }
    }

    fn delete(
        &mut self,
        request: DeleteReq,
    ) -> Result<WriteResp, DriverError> {
        match request {
            DeleteReq::Document { collection, filter } => {
                let (db, coll) = self.resolve_collection(&collection)?;
                let filter_doc =
                    to_document(&filter).map_err(|err| DriverError::Other(format!("解析过滤条件失败: {}", err)))?;
                let result = self
                    .client
                    .database(db)
                    .collection::<Document>(coll)
                    .delete_many(filter_doc)
                    .run()
                    .map_err(|err| DriverError::Other(format!("删除失败: {}", err)))?;
                Ok(WriteResp {
                    affected: result.deleted_count,
                })
            }
            other => Err(DriverError::InvalidField(format!(
                "MongoDB 删除仅支持文档操作，收到: {:?}",
                other
            ))),
        }
    }
}

impl DatabaseDriver for MongoDBDriver {
    type Config = MongoDBOptions;

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        let client = build_client(config)?;
        let database_name = default_database(config);
        client
            .database(&database_name)
            .run_command(doc! { "ping": 1 })
            .run()
            .map_err(|err| DriverError::Other(format!("ping 失败: {}", err)))?;
        Ok(())
    }

    fn create_connection(
        &self,
        config: &Self::Config,
    ) -> Result<Box<dyn DatabaseSession>, DriverError> {
        let client = build_client(config)?;
        let database_name = default_database(config);
        Ok(Box::new(MongoConnection::new(client, database_name)))
    }
}

fn build_client(config: &MongoDBOptions) -> Result<Client, DriverError> {
    let uri = build_uri(config)?;
    Client::with_uri_str(&uri).map_err(|err| DriverError::Other(format!("连接失败: {}", err)))
}

fn default_database(config: &MongoDBOptions) -> String {
    config
        .auth_source
        .as_deref()
        .filter(|name| !name.is_empty())
        .unwrap_or("admin")
        .to_string()
}

fn build_uri(config: &MongoDBOptions) -> Result<String, DriverError> {
    if let Some(uri) = &config.connection_string {
        let trimmed = uri.trim();
        if trimmed.is_empty() {
            return Err(DriverError::InvalidField("connection_string".into()));
        }
        return Ok(trimmed.to_string());
    }

    if config.hosts.is_empty() {
        return Err(DriverError::MissingField("hosts".into()));
    }

    let mut uri = String::from("mongodb://");

    if let Some(username) = &config.username {
        let username = username.trim();
        if username.is_empty() {
            return Err(DriverError::InvalidField("username".into()));
        }
        uri.push_str(username);
        if let Some(password) = &config.password {
            uri.push(':');
            uri.push_str(password);
        }
        uri.push('@');
    } else if config.password.is_some() {
        return Err(DriverError::InvalidField("username".into()));
    }

    let hosts = config
        .hosts
        .iter()
        .map(|MongoDBHost { host, port }| {
            let host = host.trim();
            if host.is_empty() {
                Err(DriverError::InvalidField("host".into()))
            } else {
                Ok(format!("{}:{}", host, port))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    uri.push_str(&hosts.join(","));

    let mut params = Vec::new();
    if let Some(auth) = &config.auth_source {
        if !auth.is_empty() {
            params.push(format!("authSource={}", auth));
        }
    }
    if let Some(rs) = &config.replica_set {
        if !rs.is_empty() {
            params.push(format!("replicaSet={}", rs));
        }
    }
    if config.tls {
        params.push("tls=true".to_string());
    }

    if !params.is_empty() {
        uri.push('/');
        uri.push('?');
        uri.push_str(&params.join("&"));
    }

    Ok(uri)
}
