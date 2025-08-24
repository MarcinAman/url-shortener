use rand::Rng;
use rand::rngs::SmallRng;

/// Generates a shortened URL by combining a checksum of the original URL with a random part
/// Hashing takes care of most of the collisions, but we still need to generate a random part to avoid collisions since CRC32 is not a secure hash function
/// We accept that if the url is the same, the shortened url will be different because of the random part. We trade it for sake of analytics
pub async fn get_shortened_url(url: String, server_domain: &str, random_part: String) -> String {
    let checksum = crc32fast::hash(url.as_bytes());
    let encoded = base62::encode(checksum);
    // We store old url and the order integer in redis using HSET command
    // We can use the TTL feature for expiration date
    format!("{}/{}{}", server_domain, encoded, random_part)
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
        let test_domain = "https://short.ly";
        let test_random_part = "abc123".to_string();

        // Act - use tokio::runtime to run the async function
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_shortened_url(test_url.clone(), test_domain, test_random_part.clone()));

        // Assert
        assert!(result.starts_with(test_domain));
        assert!(result.contains(&test_random_part));
        
        let parts: Vec<&str> = result.split('/').collect();
        // The last part should contain both checksum and random part concatenated
        let combined_part = parts[3];
        assert!(!combined_part.is_empty());
        assert!(combined_part.chars().all(|c| c.is_alphanumeric()));
        assert!(combined_part.ends_with(&test_random_part));
        
        // Verify the overall format: domain/checksum + random_part
        assert_eq!(result, format!("{}/{}", test_domain, combined_part));
        assert_eq!(result, "https://short.ly/2dHrrEabc123");
    }

    #[test]
    fn test_generate_random_code() {
        let mut rng = SmallRng::from_os_rng();
        let code = generate_random_code(&mut rng);
        
        assert!(!code.is_empty());
        assert!(code.chars().all(|c| c.is_alphanumeric()));
    }
}
