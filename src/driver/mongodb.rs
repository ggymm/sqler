use mongodb::{
    bson::{Document, doc, to_document},
    sync::Client,
};

use crate::model::{ColumnInfo, ColumnKind, MongoDBHost, MongoDBOptions, TableInfo};

use super::{DatabaseDriver, DatabaseSession, DocumentOp, DriverError, ExecReq, ExecResp, QueryReq, QueryResp};

#[derive(Debug, Clone, Copy)]
pub struct MongoDBDriver;

impl DatabaseDriver for MongoDBDriver {
    type Config = MongoDBOptions;

    fn supp_kinds(&self) -> Vec<ColumnKind> {
        vec![
            ColumnKind::Int,
            ColumnKind::BigInt,
            ColumnKind::Double,
            ColumnKind::String,
            ColumnKind::Boolean,
            ColumnKind::Date,
            ColumnKind::Timestamp,
            ColumnKind::Document,
            ColumnKind::Array,
            ColumnKind::Binary,
        ]
    }

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        let client = open_conn(config)?;
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
        let client = open_conn(config)?;
        let database_name = default_database(config);
        Ok(Box::new(MongoSession::new(client, database_name)))
    }
}

struct MongoSession {
    client: Client,
    default_db: String,
}

impl MongoSession {
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
        match full_name.split_once('.') {
            Some((db, coll)) if !db.is_empty() && !coll.is_empty() => Ok((db, coll)),
            Some(_) => Err(DriverError::InvalidField("collection".into())),
            None => {
                if self.default_db.is_empty() {
                    return Err(DriverError::InvalidField(
                        "collection 需要包含数据库前缀，例如 db.collection".into(),
                    ));
                }
                Ok((self.default_db.as_str(), full_name))
            }
        }
    }
}

impl DatabaseSession for MongoSession {
    fn exec(
        &mut self,
        req: ExecReq,
    ) -> Result<ExecResp, DriverError> {
        match req {
            ExecReq::Document { collection, operation } => {
                let (db, coll) = self.resolve_collection(&collection)?;
                match operation {
                    DocumentOp::Insert { document } => {
                        let doc = to_document(&document)
                            .map_err(|err| DriverError::Other(format!("解析文档失败: {}", err)))?;
                        self.client
                            .database(db)
                            .collection::<Document>(coll)
                            .insert_one(doc)
                            .run()
                            .map_err(|err| DriverError::Other(format!("插入失败: {}", err)))?;
                        Ok(ExecResp { affected: 1 })
                    }
                    DocumentOp::Update { filter, update } => {
                        let filter_doc = to_document(&filter)
                            .map_err(|err| DriverError::Other(format!("解析过滤条件失败: {}", err)))?;
                        let update_doc = to_document(&update)
                            .map_err(|err| DriverError::Other(format!("解析更新内容失败: {}", err)))?;

                        let result = self
                            .client
                            .database(db)
                            .collection::<Document>(coll)
                            .update_many(filter_doc, update_doc)
                            .run()
                            .map_err(|err| DriverError::Other(format!("更新失败: {}", err)))?;
                        Ok(ExecResp {
                            affected: result.modified_count,
                        })
                    }
                    DocumentOp::Delete { filter } => {
                        let filter_doc = to_document(&filter)
                            .map_err(|err| DriverError::Other(format!("解析过滤条件失败: {}", err)))?;
                        let result = self
                            .client
                            .database(db)
                            .collection::<Document>(coll)
                            .delete_many(filter_doc)
                            .run()
                            .map_err(|err| DriverError::Other(format!("删除失败: {}", err)))?;
                        Ok(ExecResp {
                            affected: result.deleted_count,
                        })
                    }
                }
            }
            other => Err(DriverError::InvalidField(format!(
                "MongoDB 仅支持文档操作，收到: {:?}",
                other
            ))),
        }
    }

    fn query(
        &mut self,
        req: QueryReq,
    ) -> Result<QueryResp, DriverError> {
        match req {
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

                let mut docs = vec![];
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

    fn tables(&mut self) -> Result<Vec<TableInfo>, DriverError> {
        let db = self.client.database(&self.default_db);
        let collection_names = db
            .list_collection_names()
            .run()
            .map_err(|err| DriverError::Other(format!("查询集合列表失败: {}", err)))?;

        let tables = collection_names
            .into_iter()
            .map(|name| TableInfo {
                name,
                row_count: None,
                size_bytes: None,
                last_accessed: None,
            })
            .collect();

        Ok(tables)
    }

    fn columns(
        &mut self,
        _table: &str,
    ) -> Result<Vec<ColumnInfo>, DriverError> {
        Err(DriverError::Other("MongoDB 作为文档数据库不支持固定列结构查询".into()))
    }
}

fn open_conn(config: &MongoDBOptions) -> Result<Client, DriverError> {
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

    match (&config.username, &config.password) {
        (Some(username), password) => {
            let username = username.trim();
            if username.is_empty() {
                return Err(DriverError::InvalidField("username".into()));
            }
            uri.push_str(username);
            if let Some(password) = password {
                uri.push(':');
                uri.push_str(password);
            }
            uri.push('@');
        }
        (None, Some(_)) => {
            return Err(DriverError::InvalidField("username".into()));
        }
        (None, None) => {}
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

    let mut params = vec![];
    if let Some(auth) = &config.auth_source.as_deref().filter(|s| !s.is_empty()) {
        params.push(format!("authSource={}", auth));
    }
    if let Some(rs) = &config.replica_set.as_deref().filter(|s| !s.is_empty()) {
        params.push(format!("replicaSet={}", rs));
    }
    if config.use_tls {
        params.push("tls=true".to_string());
    }

    if !params.is_empty() {
        uri.push('/');
        uri.push('?');
        uri.push_str(&params.join("&"));
    }

    Ok(uri)
}
