pub mod client;
pub mod config;
pub mod connector;
pub mod historical_sync;
pub mod types;

pub use client::PolygonClient;
pub use config::PolygonConfig;
pub use connector::PolygonConnector;
pub use historical_sync::{get_last_synced_time, sync_polygon_history};
pub use types::{resolution_to_polygon_params, AggregateBar};
