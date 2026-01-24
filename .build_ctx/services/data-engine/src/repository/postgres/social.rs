use async_trait::async_trait;
use sqlx::PgPool;
use crate::error::DataEngineError;
use crate::models::SocialData;
use crate::repository::SocialRepository;

pub struct PostgresSocialRepository {
    pool: PgPool,
}

impl PostgresSocialRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SocialRepository for PostgresSocialRepository {
    async fn insert_tweet(&self, data: &SocialData) -> Result<(), DataEngineError> {
        sqlx::query(r#"
            INSERT INTO tweets (
                id, username, text, created_at, user_id, followers_count, verified,
                retweet_count, favorite_count, reply_count, quote_count,
                is_retweet, is_reply, hashtags, media_urls, raw_data
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (id) DO UPDATE SET
                retweet_count = EXCLUDED.retweet_count,
                favorite_count = EXCLUDED.favorite_count,
                reply_count = EXCLUDED.reply_count,
                quote_count = EXCLUDED.quote_count
        "#)
        .bind(data.id)
        .bind(&data.username)
        .bind(&data.text)
        .bind(data.created_at)
        .bind(data.user_id)
        .bind(data.followers_count)
        .bind(data.verified)
        .bind(data.retweet_count)
        .bind(data.favorite_count)
        .bind(data.reply_count)
        .bind(data.quote_count)
        .bind(data.is_retweet)
        .bind(data.is_reply)
        .bind(&data.hashtags)
        .bind(&data.media_urls)
        .bind(&data.raw_data)
        .execute(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to insert tweet: {}", e)))?;
        Ok(())
    }

    async fn insert_collection_run(&self, target: &str, scraped: i32, upserted: i32, error: Option<&str>) -> Result<(), DataEngineError> {
         sqlx::query(r#"
            INSERT INTO twitter_collection_runs (target, scraped_count, upserted_count, error)
            VALUES ($1, $2, $3, $4)
         "#)
         .bind(target)
         .bind(scraped)
         .bind(upserted)
         .bind(error)
         .execute(&self.pool)
         .await
         .map_err(|e| DataEngineError::DatabaseError(format!("Failed to insert run: {}", e)))?;
         Ok(())
    }
}
