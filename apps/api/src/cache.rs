use redis::AsyncCommands;
use tracing::{debug, info, warn};

pub mod keys {
    pub fn product_page(handle: &str) -> String {
        format!("storefront:product-page:{}", handle)
    }
}

pub const PRODUCT_PAGE_TTL: u64 = 300;

#[derive(Clone)]
pub struct CacheClient {
    client: redis::Client,
}

impl CacheClient {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn cache_get(&self, key: &str) -> Option<String> {
        let mut conn = match self.client.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                warn!(cache_key = key, error = %e, "redis get failed — cache miss");
                return None;
            }
        };

        match conn.get::<_, Option<String>>(key).await {
            Ok(Some(value)) => {
                debug!(cache_key = key, "cache hit");
                Some(value)
            }
            Ok(None) => {
                info!(cache_key = key, "cache miss");
                None
            }
            Err(e) => {
                warn!(cache_key = key, error = %e, "redis get failed — cache miss");
                None
            }
        }
    }

    pub async fn cache_set(&self, key: &str, value: &str, ttl_secs: u64) {
        let mut conn = match self.client.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                warn!(cache_key = key, error = %e, "cache set failed");
                return;
            }
        };

        match conn.set_ex::<_, _, ()>(key, value, ttl_secs).await {
            Ok(_) => {
                debug!(cache_key = key, ttl = ttl_secs, "cache set");
            }
            Err(e) => {
                warn!(cache_key = key, error = %e, "cache set failed");
            }
        }
    }

    pub async fn cache_delete(&self, key: &str) {
        let mut conn = match self.client.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                warn!(cache_key = key, error = %e, "cache delete failed");
                return;
            }
        };

        match conn.del::<_, ()>(key).await {
            Ok(_) => {
                debug!(cache_key = key, "cache deleted");
            }
            Err(e) => {
                warn!(cache_key = key, error = %e, "cache delete failed");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_set_and_get() {
        let client =
            CacheClient::new("redis://127.0.0.1:6379").expect("Failed to connect to redis");
        let key = "test:set_get";
        let value = "test_value";

        client.cache_set(key, value, 10).await;
        let result = client.cache_get(key).await;
        assert_eq!(result, Some(value.to_string()));

        client.cache_delete(key).await;
    }

    #[tokio::test]
    async fn test_cache_get_miss() {
        let client =
            CacheClient::new("redis://127.0.0.1:6379").expect("Failed to connect to redis");
        let key = "test:miss";

        client.cache_delete(key).await;
        let result = client.cache_get(key).await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_cache_delete() {
        let client =
            CacheClient::new("redis://127.0.0.1:6379").expect("Failed to connect to redis");
        let key = "test:delete";
        let value = "test_value";

        client.cache_set(key, value, 10).await;
        client.cache_delete(key).await;
        let result = client.cache_get(key).await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_cache_get_invalid_redis_returns_none() {
        let client = CacheClient::new("redis://127.0.0.1:9999").unwrap();
        let key = "test:invalid_redis";
        let result = client.cache_get(key).await;
        assert_eq!(result, None);
    }
}
