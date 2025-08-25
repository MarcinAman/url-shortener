use rand::rngs::SmallRng;
use rand::Rng;

/// Generates a shortened URL by combining a checksum of the original URL with a random part
/// Hashing takes care of most of the collisions, but we still need to generate a random part to avoid collisions since CRC32 is not a secure hash function
/// We accept that if the url is the same, the shortened url will be different because of the random part. We trade it for sake of analytics
pub async fn get_url_slug(url: String, random_part: Option<String>) -> String {
    let checksum = crc32fast::hash(url.as_bytes());
    let encoded = base62::encode(checksum);
    // We store old url and the order integer in redis using HSET command
    // We can use the TTL feature for expiration date
    format!("{}{}", encoded, random_part.unwrap_or("".to_string()))
}

/// Generates a random code using base62 encoding
pub fn generate_random_code(rng: &mut SmallRng) -> String {
    let random_number: u32 = rng.random();
    base62::encode(random_number)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_get_shortened_url_happy_path() {
        // Arrange
        let test_url = "https://example.com/very/long/url/that/needs/shortening".to_string();
        let test_random_part = "abc123".to_string();

        // Act - use tokio::runtime to run the async function
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_url_slug(test_url.clone(), Some(test_random_part.clone())));

        // Assert
        assert!(result.contains(&test_random_part));

        assert!(!result.is_empty());
        assert!(result.chars().all(|c| c.is_alphanumeric()));
        assert!(result.ends_with(&test_random_part));
    }

    #[test]
    fn test_generate_random_code() {
        let mut rng = SmallRng::from_os_rng();
        let code = generate_random_code(&mut rng);

        assert!(!code.is_empty());
        assert!(code.chars().all(|c| c.is_alphanumeric()));
    }
}
