use std::collections::{HashMap, HashSet};
use std::env;
use std::time::Duration;

use tokio_postgres::Client as PgClient;
use tracing::{error, info, warn};

use crate::traders::BrokerPosition;

/// Maps IBKR account IDs (e.g. "DUxxxxxxx") → internal account_ids (e.g. "ibkr_long_only").
/// Built from IBKR_ACCOUNT_LONG_ONLY / IBKR_ACCOUNT_LONG_SHORT env vars.
fn build_account_map() -> HashMap<String, String> {
    let mut map = HashMap::new();

    if let Ok(acct) = env::var("IBKR_ACCOUNT_LONG_ONLY") {
        if !acct.is_empty() {
            map.insert(acct, "ibkr_long_only".to_string());
        }
    }
    if let Ok(acct) = env::var("IBKR_ACCOUNT_LONG_SHORT") {
        if !acct.is_empty() {
            map.insert(acct, "ibkr_long_short".to_string());
        }
    }

    map
}

/// Resolve an IBKR account ID to an internal account_id.
/// Falls back to "ibkr_{ibkr_account}" if not mapped.
fn resolve_account_id(ibkr_account: &str, map: &HashMap<String, String>) -> String {
    map.get(ibkr_account)
        .cloned()
        .unwrap_or_else(|| format!("ibkr_{}", ibkr_account))
}

/// Sync IBKR positions → trade_positions table.
/// Accepts pre-fetched positions to avoid redundant API calls.
///
/// Defensive behavior:
/// - If IBKR returns 0 positions, skip entirely (treats empty as API anomaly)
/// - Only delete DB positions for accounts that appeared in the IBKR response
/// - 5-minute grace period: recently-updated positions are not deleted
pub async fn sync_positions(positions: &[BrokerPosition], db: &PgClient) {
    // Defensive: if IBKR returned nothing, treat as API anomaly — skip sync entirely
    if positions.is_empty() {
        warn!("Position sync: IBKR returned 0 positions — skipping (possible API issue)");
        return;
    }

    let account_map = build_account_map();

    // Build map of what IBKR has: (account_id, symbol) → (qty, avg_cost)
    // Track which accounts were seen in the response
    let mut ibkr_map: HashMap<(String, String), (f64, f64)> = HashMap::new();
    let mut accounts_seen: HashSet<String> = HashSet::new();

    for pos in positions {
        let account_id = resolve_account_id(&pos.account, &account_map);
        accounts_seen.insert(account_id.clone());
        if pos.quantity.abs() < 1e-9 {
            continue; // Skip zero-quantity positions
        }
        ibkr_map.insert(
            (account_id, pos.symbol.clone()),
            (pos.quantity, pos.avg_cost),
        );
    }

    // Get current DB positions (cast numeric → float8 to avoid rust_decimal dependency)
    let db_rows = match db
        .query(
            "SELECT account_id, symbol, quantity::float8, avg_price::float8 FROM trade_positions",
            &[],
        )
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            error!("Position sync: failed to query trade_positions: {}", e);
            return;
        }
    };

    let mut db_map: HashMap<(String, String), (f64, f64)> = HashMap::new();
    for row in &db_rows {
        let account_id: String = row.get(0);
        let symbol: String = row.get(1);
        let qty: f64 = row.get(2);
        let avg_price: f64 = row.get(3);
        db_map.insert((account_id, symbol), (qty, avg_price));
    }

    let mut inserts = 0u32;
    let mut updates = 0u32;
    let mut deletes = 0u32;

    // Positions in IBKR but not in DB, or with different quantity → upsert
    for ((account_id, symbol), (ibkr_qty, ibkr_avg)) in &ibkr_map {
        match db_map.get(&(account_id.clone(), symbol.clone())) {
            Some((db_qty, _)) if (db_qty - ibkr_qty).abs() < 1e-6 => {
                // Quantities match — no action needed
            }
            Some((db_qty, _)) => {
                // Quantity mismatch — update to IBKR value
                warn!(
                    "Position sync: {} {} qty mismatch: DB={} IBKR={} → updating",
                    account_id, symbol, db_qty, ibkr_qty
                );
                let res = db
                    .execute(
                        "UPDATE trade_positions SET quantity = $1::float8, avg_price = $2::float8, updated_at = NOW()
                     WHERE account_id = $3 AND symbol = $4",
                        &[ibkr_qty, ibkr_avg, account_id, symbol],
                    )
                    .await;
                if let Err(e) = res {
                    error!(
                        "Position sync: failed to update {} {}: {}",
                        account_id, symbol, e
                    );
                } else {
                    updates += 1;
                }
            }
            None => {
                // New position in IBKR, not in DB → insert
                info!(
                    "Position sync: new position {} {} qty={} avg={} → inserting",
                    account_id, symbol, ibkr_qty, ibkr_avg
                );
                let res = db
                    .execute(
                        "INSERT INTO trade_positions (account_id, exchange, symbol, quantity, avg_price, updated_at)
                     VALUES ($1, 'polygon', $2, $3::float8, $4::float8, NOW())
                     ON CONFLICT (account_id, exchange, symbol) DO UPDATE
                     SET quantity = $3::float8, avg_price = $4::float8, updated_at = NOW()",
                        &[account_id, symbol, ibkr_qty, ibkr_avg],
                    )
                    .await;
                if let Err(e) = res {
                    error!(
                        "Position sync: failed to insert {} {}: {}",
                        account_id, symbol, e
                    );
                } else {
                    inserts += 1;
                }
            }
        }
    }

    // Positions in DB but not in IBKR → delete (position was closed at broker)
    // Defensive: only delete for accounts that appeared in the IBKR response.
    // If an account returned no data, we assume API issue and skip deletions for it.
    // Grace period: skip positions updated within the last 5 minutes.
    for key in db_map.keys() {
        if !key.0.starts_with("ibkr") {
            continue; // Only reconcile IBKR accounts
        }
        // Only delete if this account was seen in the IBKR response
        if !accounts_seen.contains(&key.0) {
            continue;
        }
        if !ibkr_map.contains_key(key) {
            let res = db
                .execute(
                    "DELETE FROM trade_positions \
                     WHERE account_id = $1 AND symbol = $2 \
                     AND updated_at < NOW() - INTERVAL '5 minutes'",
                    &[&key.0, &key.1],
                )
                .await;
            match res {
                Ok(0) => {
                    info!(
                        "Position sync: {} {} not in IBKR but recently updated → skipping (grace period)",
                        key.0, key.1
                    );
                }
                Ok(n) => {
                    info!(
                        "Position sync: {} {} no longer in IBKR → deleted ({} row)",
                        key.0, key.1, n
                    );
                    deletes += n as u32;
                }
                Err(e) => {
                    error!(
                        "Position sync: failed to delete {} {}: {}",
                        key.0, key.1, e
                    );
                }
            }
        }
    }

    if inserts > 0 || updates > 0 || deletes > 0 {
        info!(
            "Position sync: {} inserted, {} updated, {} deleted (IBKR: {} positions, {} accounts)",
            inserts, updates, deletes, positions.len(), accounts_seen.len()
        );
    }
}

/// Mark stale "Submitted"/"PreSubmitted" orders that are older than `max_age` as "StaleFill".
/// These orders likely filled at IBKR but we missed the notification.
pub async fn mark_stale_orders(db: &PgClient, max_age: Duration) {
    let age_secs = max_age.as_secs() as i64;

    let res = db
        .execute(
            "UPDATE trade_orders
         SET status = 'StaleFill', updated_at = NOW()
         WHERE status IN ('Submitted', 'PreSubmitted')
         AND created_at < NOW() - make_interval(secs => $1::float8)
         AND exchange != 'test'",
            &[&(age_secs as f64)],
        )
        .await;

    match res {
        Ok(count) if count > 0 => {
            warn!(
                "Reconciliation: marked {} stale orders as StaleFill (older than {}s)",
                count, age_secs
            );
        }
        Err(e) => {
            error!("Reconciliation: failed to mark stale orders: {}", e);
        }
        _ => {}
    }
}
