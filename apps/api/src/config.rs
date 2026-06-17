#[derive(Clone)]
pub struct Config {
    pub port: u16,
}

impl Config {
    pub fn load() -> Self {
        // Later, we will load this from environment variables
        Self { port: 3000 }
    }
}
