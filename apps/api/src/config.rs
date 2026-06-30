#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
}

impl Config {
    pub fn load() -> Self {
        // Load the .env file if it exists
        let _ = dotenvy::dotenv();

        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);

        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://ahlan:ahlan_dev@localhost:5432/ahlan_commerce".to_string()
        });

        Self { port, database_url }
    }
}
