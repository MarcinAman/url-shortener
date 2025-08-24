use redis::{Client, RedisError, aio::ConnectionManager};
use std::sync::Arc;

pub struct RedisService {
    connection_manager: Arc<ConnectionManager>,
}

impl RedisService {
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        let connection_manager = ConnectionManager::new(client).await?;
        
        Ok(RedisService {
            connection_manager: Arc::new(connection_manager),
        })
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>, RedisError> {
        let mut conn = (*self.connection_manager).clone();
        redis::cmd("GET").arg(key).query_async(&mut conn).await
    }

    pub async fn set(&self, key: &str, value: &str, ttl: Option<usize>) -> Result<(), RedisError> {
        let mut conn = (*self.connection_manager).clone();
        
        if let Some(ttl_seconds) = ttl {
            redis::cmd("SET").arg(key).arg(value).arg("EX").arg(ttl_seconds).arg("NX").query_async(&mut conn).await
        } else {
            redis::cmd("SET").arg(key).arg(value).arg("NX").query_async(&mut conn).await
        }
    }
}

pub async fn get_redis_service() -> Result<RedisService, RedisError> {
    RedisService::new("redis://localhost:6379").await
}
