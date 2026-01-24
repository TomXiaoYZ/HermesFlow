use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Social media data (Twitter/X)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialData {
    /// Tweet ID
    pub id: i64,
    /// Username
    pub username: String,
    /// Tweet text content
    pub text: String,
    /// When the tweet was created
    pub created_at: DateTime<Utc>,
    /// When we received the data
    pub received_at: DateTime<Utc>,

    // User metadata
    pub user_id: Option<i64>,
    pub followers_count: Option<i32>, // Match DB field name
    pub verified: bool,               // Match DB field name

    // Engagement metrics
    pub retweet_count: i32,
    pub favorite_count: i32,
    pub reply_count: i32,
    pub quote_count: i32,

    // Tweet characteristics
    pub is_retweet: bool,
    pub is_reply: bool,
    pub hashtags: Vec<String>,
    pub media_urls: Vec<String>,

    // Raw JSON for debugging
    pub raw_data: serde_json::Value,
}

impl SocialData {
    /// Creates a new SocialData instance
    pub fn new(id: i64, username: String, text: String, created_at: DateTime<Utc>) -> Self {
        Self {
            id,
            username,
            text,
            created_at,
            received_at: Utc::now(),
            user_id: None,
            followers_count: None,
            verified: false,
            retweet_count: 0,
            favorite_count: 0,
            reply_count: 0,
            quote_count: 0,
            is_retweet: false,
            is_reply: false,
            hashtags: vec![],
            media_urls: vec![],
            raw_data: serde_json::json!({}),
        }
    }

    /// Calculates engagement score
    pub fn engagement_score(&self) -> f64 {
        (self.retweet_count as f64 * 2.0)
            + (self.favorite_count as f64)
            + (self.reply_count as f64 * 1.5)
            + (self.quote_count as f64 * 2.5)
    }

    /// Checks if this is a high-engagement tweet
    pub fn is_high_engagement(&self, threshold: i32) -> bool {
        (self.retweet_count + self.favorite_count + self.reply_count + self.quote_count) > threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_social_data_creation() {
        let data = SocialData::new(
            123456789,
            "testuser".to_string(),
            "Test tweet".to_string(),
            Utc::now(),
        );

        assert_eq!(data.id, 123456789);
        assert_eq!(data.username, "testuser");
        assert_eq!(data.text, "Test tweet");
    }

    #[test]
    fn test_engagement_score() {
        let mut data = SocialData::new(
            123456789,
            "testuser".to_string(),
            "Test tweet".to_string(),
            Utc::now(),
        );

        data.retweet_count = 10;
        data.favorite_count = 20;
        data.reply_count = 5;
        data.quote_count = 2;

        let score = data.engagement_score();
        assert!(score > 0.0);
    }

    #[test]
    fn test_high_engagement() {
        let mut data = SocialData::new(
            123456789,
            "testuser".to_string(),
            "Test tweet".to_string(),
            Utc::now(),
        );

        data.retweet_count = 100;
        data.favorite_count = 500;

        assert!(data.is_high_engagement(100));
        assert!(!data.is_high_engagement(1000));
    }
}
