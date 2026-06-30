pub mod cornucopia;
pub mod import_jobs;
pub mod products;

use chrono::{DateTime, Utc};
pub use deadpool_postgres::{Config, Pool, Runtime};
use std::str::FromStr;
use time::OffsetDateTime;

pub fn convert_time_to_chrono(t: OffsetDateTime) -> DateTime<Utc> {
    DateTime::from_timestamp(t.unix_timestamp(), t.nanosecond()).unwrap_or_default()
}

pub fn create_pool(url: &str) -> Pool {
    let config = tokio_postgres::Config::from_str(url).expect("Invalid database URL");
    let mgr = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    Pool::builder(mgr).build().expect("Failed to create pool")
}
