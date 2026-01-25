// Add helper function to load factors from database
use sqlx::PgPool;

async fn load_active_factors(pool: &PgPool) -> Result<Vec<FactorConfig>, sqlx::Error> {
    let configs = sqlx::query_as::<_, FactorConfig>(
        "SELECT slug, normalization, parameters FROM factors WHERE is_active = true ORDER BY id"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(configs)
}
