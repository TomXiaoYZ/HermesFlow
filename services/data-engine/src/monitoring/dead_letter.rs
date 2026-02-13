use crate::models::StandardMarketData;
use crate::monitoring::metrics::DEAD_LETTER_TOTAL;

/// Log a permanently dropped record after all retry attempts have been exhausted.
///
/// This serves as a structured audit trail for data loss events. The `dead_letter`
/// tracing target allows filtering these events independently in log aggregation.
pub fn log_dead_letter(data: &StandardMarketData, error: &str, target: &str) {
    DEAD_LETTER_TOTAL.inc();
    tracing::error!(
        target: "dead_letter",
        symbol = %data.symbol,
        source = %data.source,
        price = %data.price,
        timestamp = data.timestamp,
        storage_target = target,
        error = error,
        "DEAD LETTER: data permanently dropped after retry exhaustion"
    );
}
