use redis::{Client, Connection, Value};
use serde_json::{Map as JsonMap, Number, Value as JsonValue};

use super::{
    number_from_f64, DatabaseDriver, DatabaseSession, DeleteReq, DriverError, InsertReq, QueryReq, QueryResp,
    UpdateReq, WriteResp,
};
use crate::option::RedisOptions;

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
                Ok(QueryResp::Value(redis_value_to_json(value)))
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
    ) -> Result<WriteResp, DriverError> {
        match request {
            InsertReq::Command { name, args } => self.exec_write(&name, &args),
            other => Err(DriverError::InvalidField(format!(
                "Redis 插入仅支持命令，收到: {:?}",
                other
            ))),
        }
    }

    fn update(
        &mut self,
        request: UpdateReq,
    ) -> Result<WriteResp, DriverError> {
        match request {
            UpdateReq::Command { name, args } => self.exec_write(&name, &args),
            other => Err(DriverError::InvalidField(format!(
                "Redis 更新仅支持命令，收到: {:?}",
                other
            ))),
        }
    }

    fn delete(
        &mut self,
        request: DeleteReq,
    ) -> Result<WriteResp, DriverError> {
        match request {
            DeleteReq::Command { name, args } => self.exec_write(&name, &args),
            other => Err(DriverError::InvalidField(format!(
                "Redis 删除仅支持命令，收到: {:?}",
                other
            ))),
        }
    }
}

impl RedisConnection {
    fn exec_write(
        &mut self,
        name: &str,
        args: &[JsonValue],
    ) -> Result<WriteResp, DriverError> {
        let value = execute(&mut self.conn, name, args)?;
        Ok(WriteResp {
            affected: redis_value_to_affected(&value),
        })
    }
}

impl DatabaseDriver for RedisDriver {
    type Config = RedisOptions;

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        let mut conn = open_connection(config)?;

        redis::cmd("SELECT")
            .arg(config.db as i64)
            .query::<()>(&mut conn)
            .map_err(|err| DriverError::Other(format!("选择数据库失败: {}", err)))?;

        redis::cmd("PING")
            .query::<String>(&mut conn)
            .map_err(|err| DriverError::Other(format!("PING 失败: {}", err)))?;

        Ok(())
    }

    fn create_connection(
        &self,
        config: &Self::Config,
    ) -> Result<Box<dyn DatabaseSession>, DriverError> {
        let conn = open_connection(config)?;
        Ok(Box::new(RedisConnection::new(conn)))
    }
}

fn open_connection(config: &RedisOptions) -> Result<Connection, DriverError> {
    if config.host.trim().is_empty() {
        return Err(DriverError::MissingField("host".into()));
    }
    if config.port == 0 {
        return Err(DriverError::InvalidField("port".into()));
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

    if let Some(username) = &config.username {
        let username = username.trim();
        if username.is_empty() {
            return Err(DriverError::InvalidField("username".into()));
        }
        url.push_str(username);
        if let Some(password) = &config.password {
            url.push(':');
            url.push_str(password);
        }
        url.push('@');
    } else if let Some(password) = &config.password {
        url.push(':');
        url.push_str(password);
        url.push('@');
    }

    url.push_str(config.host.trim());
    url.push(':');
    url.push_str(&config.port.to_string());
    url.push('/');
    url.push_str(&config.db.to_string());

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

fn redis_value_to_json(value: Value) -> JsonValue {
    match value {
        Value::Nil => JsonValue::Null,
        Value::Int(v) => JsonValue::Number(Number::from(v)),
        Value::BulkString(bytes) => JsonValue::String(String::from_utf8_lossy(&bytes).into_owned()),
        Value::Array(values) => JsonValue::Array(values.into_iter().map(redis_value_to_json).collect()),
        Value::SimpleString(text) => JsonValue::String(text),
        Value::Okay => JsonValue::String("OK".into()),
        Value::Map(pairs) => {
            let mut map = JsonMap::new();
            for (key, value) in pairs {
                map.insert(redis_value_key(&key), redis_value_to_json(value));
            }
            JsonValue::Object(map)
        }
        Value::Attribute { data, attributes } => {
            let mut map = JsonMap::new();
            map.insert("data".into(), redis_value_to_json(*data));
            let mut attrs = JsonMap::new();
            for (key, value) in attributes {
                attrs.insert(redis_value_key(&key), redis_value_to_json(value));
            }
            map.insert("attributes".into(), JsonValue::Object(attrs));
            JsonValue::Object(map)
        }
        Value::Set(values) => JsonValue::Array(values.into_iter().map(redis_value_to_json).collect()),
        Value::Double(v) => number_from_f64(v),
        Value::Boolean(v) => JsonValue::Bool(v),
        Value::VerbatimString { text, .. } => JsonValue::String(text),
        Value::BigNumber(value) => JsonValue::String(format!("{:?}", value)),
        Value::Push { kind, data } => {
            let mut map = JsonMap::new();
            map.insert("kind".into(), JsonValue::String(format!("{:?}", kind)));
            map.insert(
                "data".into(),
                JsonValue::Array(data.into_iter().map(redis_value_to_json).collect()),
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
