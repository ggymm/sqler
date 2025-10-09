use iced::Task;
use mongodb::sync::{Client, Collection};
use mongodb::{bson, bson::Bson};
use serde_json::{Map as JsonMap, Value as JsonValue};

use super::{
    ConnectionParams, DriverError, ExecuteRequest, ExecuteResponse, KeyAction, KeyCommand, KeyQuery, QueryRequest,
    QueryResponse, execution_response, invalid_request, make_document_response, unsupported,
};

#[derive(Clone, Debug)]
pub struct MongoDriver;

impl MongoDriver {
    pub fn new() -> Self {
        Self
    }

    pub fn test_connection(
        &self,
        params: ConnectionParams,
    ) -> Task<Result<(), DriverError>> {
        Task::future(async move {
            let client = Self::client(&params)?;
            client
                .database("admin")
                .run_command(bson::doc! { "ping": 1 })
                .run()
                .map_err(|err| DriverError::Connection(err.to_string()))?;
            Ok(())
        })
    }

    pub fn query(
        &self,
        params: ConnectionParams,
        request: QueryRequest,
    ) -> Task<Result<QueryResponse, DriverError>> {
        Task::future(async move {
            match request {
                QueryRequest::KeyValue(spec) => Self::execute_query(&params, spec),
                QueryRequest::Sql { .. } => Err(unsupported("MongoDB 不支持 SQL 查询")),
            }
        })
    }

    pub fn execute(
        &self,
        params: ConnectionParams,
        request: ExecuteRequest,
    ) -> Task<Result<ExecuteResponse, DriverError>> {
        Task::future(async move {
            match request {
                ExecuteRequest::KeyValue(cmd) => Self::execute_command(&params, cmd),
                ExecuteRequest::Sql { .. } => Err(unsupported("MongoDB 不支持 SQL 执行")),
            }
        })
    }

    fn execute_query(
        params: &ConnectionParams,
        query: KeyQuery,
    ) -> Result<QueryResponse, DriverError> {
        let client = Self::client(params)?;
        let collection = Self::collection(&client, &query.database, &query.collection)?;

        let filter = json_to_document(query.filter)?;
        let mut find = collection.find(filter);

        if let Some(limit) = query.limit {
            find = find.limit(limit);
        }

        if let Some(skip) = query.skip {
            find = find.skip(skip);
        }

        if let Some(projection) = query.projection {
            find = find.projection(json_to_document(projection)?);
        }

        if let Some(sort) = query.sort {
            find = find.sort(json_to_document(sort)?);
        }

        let mut cursor = find.run().map_err(|err| DriverError::Query(err.to_string()))?;

        let mut documents = Vec::new();
        while let Some(doc) = cursor.next() {
            let doc = doc.map_err(|err| DriverError::Query(err.to_string()))?;
            documents.push(serde_json::to_value(&doc).map_err(|err| DriverError::Query(err.to_string()))?);
        }

        Ok(make_document_response(documents))
    }

    fn execute_command(
        params: &ConnectionParams,
        cmd: KeyCommand,
    ) -> Result<ExecuteResponse, DriverError> {
        let client = Self::client(params)?;
        let collection = Self::collection(&client, &cmd.database, &cmd.collection)?;

        match cmd.action {
            KeyAction::InsertOne { document } => {
                let doc = json_to_document(document)?;
                let result = collection
                    .insert_one(doc)
                    .run()
                    .map_err(|err| DriverError::Execution(err.to_string()))?;

                let generated_value = bson_to_json(result.inserted_id)?;
                let generated = match generated_value {
                    JsonValue::Null => None,
                    other => Some(other),
                };

                Ok(execution_response(1, generated))
            }
            KeyAction::UpdateMany { filter, update } => {
                let filter = json_to_document(filter)?;
                let update = json_to_document(update)?;
                let result = collection
                    .update_many(filter, update)
                    .run()
                    .map_err(|err| DriverError::Execution(err.to_string()))?;

                let mut meta = JsonMap::new();
                meta.insert("matched_count".into(), JsonValue::from(result.matched_count));
                meta.insert("modified_count".into(), JsonValue::from(result.modified_count));
                if let Some(upserted_id) = result.upserted_id {
                    meta.insert("upserted_id".into(), bson_to_json(upserted_id)?);
                }

                Ok(execution_response(result.modified_count, Some(JsonValue::Object(meta))))
            }
            KeyAction::DeleteMany { filter } => {
                let filter = json_to_document(filter)?;
                let result = collection
                    .delete_many(filter)
                    .run()
                    .map_err(|err| DriverError::Execution(err.to_string()))?;

                Ok(execution_response(result.deleted_count, None))
            }
        }
    }

    fn client(params: &ConnectionParams) -> Result<Client, DriverError> {
        let uri = params.require_connection_string()?.to_string();
        Client::with_uri_str(&uri).map_err(|err| DriverError::Connection(err.to_string()))
    }

    fn collection(
        client: &Client,
        database: &str,
        collection: &str,
    ) -> Result<Collection<bson::Document>, DriverError> {
        if database.is_empty() {
            return Err(invalid_request("MongoDB 查询需要指定数据库名称"));
        }
        if collection.is_empty() {
            return Err(invalid_request("MongoDB 查询需要指定集合名称"));
        }

        Ok(client.database(database).collection(collection))
    }
}

fn json_to_document(value: JsonValue) -> Result<bson::Document, DriverError> {
    if value.is_null() {
        return Ok(bson::Document::new());
    }
    match bson::to_bson(&value) {
        Ok(Bson::Document(doc)) => Ok(doc),
        Ok(_) => Err(invalid_request("需要一个对象类型的参数")),
        Err(err) => Err(invalid_request(err.to_string())),
    }
}

fn bson_to_json(bson: Bson) -> Result<JsonValue, DriverError> {
    serde_json::to_value(bson).map_err(|err| DriverError::Execution(err.to_string()))
}
