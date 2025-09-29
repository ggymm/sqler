#[cfg(feature = "with-mongodb")]
use crate::error::*;
#[cfg(feature = "with-mongodb")]
use mongodb::{bson, bson::Bson, bson::Document, Client, Database};

#[cfg(feature = "with-mongodb")]
pub async fn connect(uri: &str) -> Result<(Client, String)> {
    let client = Client::with_uri_str(uri).await?;
    let db = client
        .default_database()
        .ok_or_else(|| Error::InvalidInput("Mongo URI must include a default database".into()))?;
    Ok((client, db.name().to_string()))
}

#[cfg(feature = "with-mongodb")]
pub async fn ping(db: &Database) -> Result<()> {
    use mongodb::bson::doc;
    db.run_command(doc! { "ping": 1 }, None).await?;
    Ok(())
}

#[cfg(feature = "with-mongodb")]
pub async fn insert_one(db: &Database, coll: &str, json_doc: &str) -> Result<String> {
    let doc = json_to_doc(json_doc)?;
    let r = db.collection::<Document>(coll).insert_one(doc, None).await?;
    let id = r.inserted_id;
    let sv = bson_to_json_value(id)?;
    Ok(serde_json::to_string(&sv).unwrap_or("null".into()))
}

#[cfg(feature = "with-mongodb")]
pub async fn find_one(db: &Database, coll: &str, json_filter: &str) -> Result<String> {
    let filter = json_to_doc(json_filter)?;
    let doc = db
        .collection::<Document>(coll)
        .find_one(filter, None)
        .await?;
    match doc {
        Some(d) => Ok(serde_json::to_string(&bson_to_json_value(Bson::Document(d))?)
            .unwrap_or("null".into())),
        None => Ok("null".into()),
    }
}

#[cfg(feature = "with-mongodb")]
fn json_to_doc(s: &str) -> Result<Document> {
    let v: serde_json::Value = serde_json::from_str(s)
        .map_err(|e| Error::InvalidInput(format!("Invalid JSON: {e}")))?;
    let b = bson::to_bson(&v).map_err(|e| Error::InvalidInput(format!("JSON->BSON failed: {e}")))?;
    match b {
        Bson::Document(d) => Ok(d),
        _ => Err(Error::InvalidInput("JSON must be an object".into())),
    }
}

#[cfg(feature = "with-mongodb")]
fn bson_to_json_value(b: Bson) -> Result<serde_json::Value> {
    let v: serde_json::Value = bson::from_bson(b).map_err(|e| Error::Internal(format!("BSON->JSON failed: {e}")))?;
    Ok(v)
}

