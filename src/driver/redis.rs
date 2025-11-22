use redis::{Client, Connection, Value};
use serde_json::{Map as JsonMap, Number, Value as JsonValue};

use crate::model::{ColumnKind, RedisOptions};

use super::{
    DatabaseDriver, DatabaseSession, DeleteReq, DriverError, InsertReq, QueryReq, QueryResp, UpdateReq, UpdateResp,
};

#[derive(Debug, Clone, Copy)]
pub struct RedisDriver;

struct RedisConnection {
    conn: Connection,
}

impl RedisConnection {
    fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

impl DatabaseSession for RedisConnection {
    fn query(
        &mut self,
        request: QueryReq,
    ) -> Result<QueryResp, DriverError> {
        match request {
            QueryReq::Command { name, args } => {
                let value = execute(&mut self.conn, &name, &args)?;
                Ok(QueryResp::Value(parse_value(value)))
            }
            other => Err(DriverError::InvalidField(format!(
                "Redis 查询仅支持命令，收到: {:?}",
                other
            ))),
        }
    }

    fn insert(
        &mut self,
        request: InsertReq,
    ) -> Result<UpdateResp, DriverError> {
        match request {
            InsertReq::Command { name, args } => {
                let value = execute(&mut self.conn, &name, &args)?;
                Ok(UpdateResp {
                    affected: redis_value_to_affected(&value),
                })
            }
            other => Err(DriverError::InvalidField(format!(
                "Redis 插入仅支持命令，收到: {:?}",
                other
            ))),
        }
    }

    fn update(
        &mut self,
        request: UpdateReq,
    ) -> Result<UpdateResp, DriverError> {
        match request {
            UpdateReq::Command { name, args } => {
                let value = execute(&mut self.conn, &name, &args)?;
                Ok(UpdateResp {
                    affected: redis_value_to_affected(&value),
                })
            }
            other => Err(DriverError::InvalidField(format!(
                "Redis 更新仅支持命令，收到: {:?}",
                other
            ))),
        }
    }

    fn delete(
        &mut self,
        request: DeleteReq,
    ) -> Result<UpdateResp, DriverError> {
        match request {
            DeleteReq::Command { name, args } => {
                let value = execute(&mut self.conn, &name, &args)?;
                Ok(UpdateResp {
                    affected: redis_value_to_affected(&value),
                })
            }
            other => Err(DriverError::InvalidField(format!(
                "Redis 删除仅支持命令，收到: {:?}",
                other
            ))),
        }
    }

    fn tables(&mut self) -> Result<Vec<String>, DriverError> {
        Err(DriverError::Other("Redis 不支持表列表查询".into()))
    }

    fn columns(
        &mut self,
        _table: &str,
    ) -> Result<Vec<String>, DriverError> {
        Err(DriverError::Other("Redis 作为键值数据库不支持列结构查询".into()))
    }
}

impl DatabaseDriver for RedisDriver {
    type Config = RedisOptions;

    fn supp_kinds(&self) -> Vec<ColumnKind> {
        vec![ColumnKind::String, ColumnKind::List, ColumnKind::Hash, ColumnKind::ZSet]
    }

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        let mut conn = open_conn(config)?;

        redis::cmd("PING")
            .query::<String>(&mut conn)
            .map_err(|err| DriverError::Other(format!("PING 失败: {}", err)))?;

        Ok(())
    }

    fn create_connection(
        &self,
        config: &Self::Config,
    ) -> Result<Box<dyn DatabaseSession>, DriverError> {
        let conn = open_conn(config)?;
        Ok(Box::new(RedisConnection::new(conn)))
    }
}

fn open_conn(config: &RedisOptions) -> Result<Connection, DriverError> {
    if config.host.trim().is_empty() {
        return Err(DriverError::MissingField("host".into()));
    }

    let url = build_connection_url(config)?;
    let client = Client::open(url).map_err(|err| DriverError::Other(format!("创建客户端失败: {}", err)))?;
    client
        .get_connection()
        .map_err(|err| DriverError::Other(format!("建立连接失败: {}", err)))
}

fn build_connection_url(config: &RedisOptions) -> Result<String, DriverError> {
    let scheme = if config.use_tls { "rediss://" } else { "redis://" };
    let mut url = String::from(scheme);

    // 根据 username 和 password 的存在性判断认证模式
    match (&config.username, &config.password) {
        (Some(username), Some(password)) => {
            // 账号密码认证
            if username.trim().is_empty() {
                return Err(DriverError::InvalidField("username".into()));
            }
            if password.trim().is_empty() {
                return Err(DriverError::InvalidField("password".into()));
            }
            url.push_str(username.trim());
            url.push(':');
            url.push_str(password);
            url.push('@');
        }
        (None, Some(password)) => {
            // 仅密码认证
            if password.trim().is_empty() {
                return Err(DriverError::InvalidField("password".into()));
            }
            url.push(':');
            url.push_str(password);
            url.push('@');
        }
        _ => {
            // 无认证
        }
    }

    url.push_str(config.host.trim());
    url.push(':');
    url.push_str(&config.port);

    Ok(url)
}

fn execute(
    conn: &mut Connection,
    name: &str,
    args: &[JsonValue],
) -> Result<Value, DriverError> {
    let mut command = redis::cmd(name);
    for arg in args {
        command.arg(value_to_arg(arg));
    }
    command
        .query(conn)
        .map_err(|err| DriverError::Other(format!("执行命令失败: {}", err)))
}

fn value_to_arg(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => String::from("null"),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(num) => num.to_string(),
        JsonValue::String(text) => text.clone(),
        JsonValue::Array(arr) => {
            let inner = arr.iter().map(value_to_arg).collect::<Vec<_>>().join(",");
            format!("[{}]", inner)
        }
        JsonValue::Object(obj) => serde_json::to_string(obj).unwrap_or_default(),
    }
}

fn parse_value(value: Value) -> JsonValue {
    match value {
        Value::Nil => JsonValue::Null,
        Value::Int(v) => JsonValue::Number(Number::from(v)),
        Value::BulkString(bytes) => JsonValue::String(String::from_utf8_lossy(&bytes).into_owned()),
        Value::Array(values) => JsonValue::Array(values.into_iter().map(parse_value).collect()),
        Value::SimpleString(text) => JsonValue::String(text),
        Value::Okay => JsonValue::String("OK".into()),
        Value::Map(pairs) => {
            let mut map = JsonMap::new();
            for (key, value) in pairs {
                map.insert(redis_value_key(&key), parse_value(value));
            }
            JsonValue::Object(map)
        }
        Value::Attribute { data, attributes } => {
            let mut map = JsonMap::new();
            map.insert("data".into(), parse_value(*data));
            let mut attrs = JsonMap::new();
            for (key, value) in attributes {
                attrs.insert(redis_value_key(&key), parse_value(value));
            }
            map.insert("attributes".into(), JsonValue::Object(attrs));
            JsonValue::Object(map)
        }
        Value::Set(values) => JsonValue::Array(values.into_iter().map(parse_value).collect()),
        Value::Double(v) => Number::from_f64(v)
            .map(serde_json::Value::Number)
            .unwrap_or_else(|| serde_json::Value::String(v.to_string())),
        Value::Boolean(v) => JsonValue::Bool(v),
        Value::VerbatimString { text, .. } => JsonValue::String(text),
        Value::BigNumber(value) => JsonValue::String(format!("{:?}", value)),
        Value::Push { kind, data } => {
            let mut map = JsonMap::new();
            map.insert("kind".into(), JsonValue::String(format!("{:?}", kind)));
            map.insert(
                "data".into(),
                JsonValue::Array(data.into_iter().map(parse_value).collect()),
            );
            JsonValue::Object(map)
        }
        Value::ServerError(err) => JsonValue::String(err.to_string()),
        _ => JsonValue::String(format!("{:?}", value)),
    }
}

fn redis_value_key(value: &Value) -> String {
    match value {
        Value::SimpleString(text) => text.clone(),
        Value::BulkString(bytes) => String::from_utf8_lossy(bytes).into_owned(),
        Value::Int(v) => v.to_string(),
        Value::Nil => "nil".into(),
        Value::Boolean(v) => v.to_string(),
        _ => format!("{:?}", value),
    }
}

fn redis_value_to_affected(value: &Value) -> u64 {
    match value {
        Value::Int(v) if *v >= 0 => *v as u64,
        _ => 0,
    }
}
