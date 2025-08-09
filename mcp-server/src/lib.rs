pub mod search;
pub mod scrape;
pub mod types;
pub mod mcp;
pub mod rust_scraper;
pub mod stdio_service;

#[derive(Clone, Debug)]
pub struct AppState {
    pub searxng_url: String,
    pub http_client: reqwest::Client,
}

// Re-export AppState for easy access
pub use types::*;