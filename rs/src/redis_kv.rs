#[cfg(feature = "with-redis")]
use crate::error::*;
#[cfg(feature = "with-redis")]
use redis::Commands;

#[cfg(feature = "with-redis")]
pub async fn set(client: &redis::Client, key: &str, value: &[u8]) -> Result<()> {
    let client = client.clone();
    let key = key.to_string();
    let value = value.to_vec();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let mut conn = client.get_connection()?;
        conn.set::<_, _, ()>(key, value)?;
        Ok(())
    })
    .await
    .map_err(|e| Error::Internal(format!("Join error: {e}")))??;
    Ok(())
}

#[cfg(feature = "with-redis")]
pub async fn get(client: &redis::Client, key: &str) -> Result<Option<Vec<u8>>> {
    let client = client.clone();
    let key = key.to_string();
    let res = tokio::task::spawn_blocking(move || -> Result<Option<Vec<u8>>> {
        let mut conn = client.get_connection()?;
        let exists: bool = conn.exists(&key)?;
        if !exists { return Ok(None); }
        let data: Vec<u8> = conn.get(&key)?;
        Ok(Some(data))
    })
    .await
    .map_err(|e| Error::Internal(format!("Join error: {e}")))??;
    Ok(res)
}

#[cfg(feature = "with-redis")]
pub async fn ping(client: &redis::Client) -> Result<()> {
    let client = client.clone();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let mut conn = client.get_connection()?;
        let _: String = redis::cmd("PING").query(&mut conn)?;
        Ok(())
    })
    .await
    .map_err(|e| Error::Internal(format!("Join error: {e}")))??;
    Ok(())
}
