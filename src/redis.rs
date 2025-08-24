use redis::{Client, RedisError, aio::ConnectionManager};
use std::sync::Arc;

#[derive(Clone)]
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

    pub async fn set(&self, key: &str, value: &str, ttl: Option<usize>) -> Result<bool, RedisError> {
        let mut conn = (*self.connection_manager).clone();
        
        let result: Option<String> = if let Some(ttl_seconds) = ttl {
            redis::cmd("SET").arg(key).arg(value).arg("EX").arg(ttl_seconds).arg("NX").query_async(&mut conn).await?
        } else {
            redis::cmd("SET").arg(key).arg(value).arg("NX").query_async(&mut conn).await?
        };
        
        // NX returns "OK" if set was successful, nil if key already exists
        Ok(result.is_some())
    }

    /// Cleans up all data in the current Redis database (use with caution in tests)
    pub async fn cleanup(&self) -> Result<(), RedisError> {
        let mut conn = (*self.connection_manager).clone();
        redis::cmd("FLUSHDB").query_async(&mut conn).await
    }
}

pub async fn get_redis_service() -> Result<RedisService, RedisError> {
    RedisService::new("redis://localhost:6379").await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redis_service_set_then_get() {
        // Create a fresh Redis service for testing
        let redis_service = RedisService::new("redis://localhost:6379")
            .await
            .expect("Failed to connect to Redis");

        // Clean up before test
        redis_service.cleanup().await.expect("Failed to cleanup Redis");

        // Test case: set then get
        let test_key = "test_key_1";
        let test_value = "test_value_1";

        // Set the value
        let set_result = redis_service.set(test_key, test_value, Some(60 * 60 * 24)).await;
        assert!(set_result.is_ok(), "Failed to set key in Redis");
        assert!(set_result.unwrap(), "Key should have been set successfully");

        // Get the value
        let get_result = redis_service.get(test_key).await;
        assert!(get_result.is_ok(), "Failed to get key from Redis");
        
        let retrieved_value = get_result.unwrap();
        assert_eq!(retrieved_value, Some(test_value.to_string()), "Retrieved value doesn't match set value");

        // Clean up after test
        redis_service.cleanup().await.expect("Failed to cleanup Redis");
    }

    #[tokio::test]
    async fn test_redis_service_get_nonexistent_key() {
        // Create a fresh Redis service for testing
        let redis_service = RedisService::new("redis://localhost:6379")
            .await
            .expect("Failed to connect to Redis");

        // Clean up before test
        redis_service.cleanup().await.expect("Failed to cleanup Redis");

        // Test case: get for non-existent key
        let nonexistent_key = "nonexistent_key_123";

        // Try to get a non-existent key
        let get_result = redis_service.get(nonexistent_key).await;
        assert!(get_result.is_ok(), "Failed to get non-existent key from Redis");
        
        let retrieved_value = get_result.unwrap();
        assert_eq!(retrieved_value, None, "Non-existent key should return None");

        // Clean up after test
        redis_service.cleanup().await.expect("Failed to cleanup Redis");
    }

    #[tokio::test]
    async fn test_redis_service_set_nx_prevents_overwrite() {
        // Create a fresh Redis service for testing
        let redis_service = RedisService::new("redis://localhost:6379")
            .await
            .expect("Failed to connect to Redis");

        // Clean up before test
        redis_service.cleanup().await.expect("Failed to cleanup Redis");

        // Test case: set and then attempt to set again for the same key (should fail due to NX)
        let test_key = "test_key_nx";
        let first_value = "first_value";
        let second_value = "second_value";

        // Set the first value
        let first_set_result = redis_service.set(test_key, first_value, Some(60 * 60 * 24)).await;
        assert!(first_set_result.is_ok(), "Failed to set first value in Redis");
        assert!(first_set_result.unwrap(), "First key should have been set successfully");

        // Verify first value was set
        let first_get_result = redis_service.get(test_key).await;
        assert!(first_get_result.is_ok(), "Failed to get first value from Redis");
        assert_eq!(first_get_result.unwrap(), Some(first_value.to_string()));

        // Attempt to set the second value (should fail due to NX flag)
        let second_set_result = redis_service.set(test_key, second_value, Some(60 * 60 * 24)).await;
        assert!(second_set_result.is_ok(), "Second set operation should succeed but return false");
        assert!(!second_set_result.unwrap(), "Second set should return false due to NX flag preventing overwrite");

        // Verify the first value is still there (not overwritten)
        let final_get_result = redis_service.get(test_key).await;
        assert!(final_get_result.is_ok(), "Failed to get final value from Redis");
        assert_eq!(final_get_result.unwrap(), Some(first_value.to_string()), "First value should still be there");

        // Clean up after test
        redis_service.cleanup().await.expect("Failed to cleanup Redis");
    }

    #[tokio::test]
    async fn test_redis_service_ttl_functionality() {
        // Create a fresh Redis service for testing
        let redis_service = RedisService::new("redis://localhost:6379")
            .await
            .expect("Failed to connect to Redis");

        // Clean up before test
        redis_service.cleanup().await.expect("Failed to cleanup Redis");

        // Test case: set with TTL and verify expiration behavior
        let test_key = "test_key_ttl";
        let test_value = "test_value_ttl";
        let ttl_seconds: usize = 1; // Very short TTL for testing

        // Set the value with TTL
        let set_result = redis_service.set(test_key, test_value, Some(ttl_seconds)).await;
        assert!(set_result.is_ok(), "Failed to set key with TTL in Redis");
        assert!(set_result.unwrap(), "Key should have been set successfully");

        // Verify value is immediately available
        let immediate_get = redis_service.get(test_key).await;
        assert!(immediate_get.is_ok(), "Failed to get key immediately after setting");
        assert_eq!(immediate_get.unwrap(), Some(test_value.to_string()));

        // Wait for TTL to expire
        tokio::time::sleep(tokio::time::Duration::from_secs((ttl_seconds + 1) as u64)).await;

        // Verify value has expired
        let expired_get = redis_service.get(test_key).await;
        assert!(expired_get.is_ok(), "Failed to get expired key from Redis");
        assert_eq!(expired_get.unwrap(), None, "Expired key should return None");

        // Clean up after test
        redis_service.cleanup().await.expect("Failed to cleanup Redis");
    }
}
