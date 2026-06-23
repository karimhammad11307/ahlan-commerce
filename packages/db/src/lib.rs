pub mod cornucopia;
pub mod products;

pub use deadpool_postgres::{Pool, Config, Runtime};
use tokio_postgres::NoTls;
use std::str::FromStr;

pub fn create_pool(url: &str) -> Pool {
    let config = tokio_postgres::Config::from_str(url).expect("Invalid database URL");
    let mgr = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    Pool::builder(mgr).build().expect("Failed to create pool")
}
