use crate::types::*;
use crate::AppState;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

pub async fn search_web(state: &Arc<AppState>, query: &str) -> Result<Vec<SearchResult>> {
    info!("Searching for: {}", query);
    
    // Prepare search parameters
    let mut params = HashMap::new();
    params.insert("q", query);
    params.insert("format", "json");
    params.insert("engines", "duckduckgo,google,bing");
    params.insert("categories", "general");
    params.insert("time_range", "");
    params.insert("language", "en");
    params.insert("safesearch", "0");
    
    // Build search URL
    let search_url = format!("{}/search", state.searxng_url);
    debug!("Search URL: {}", search_url);
    
    // Make request to SearXNG
    let response = state
        .http_client
        .get(&search_url)
        .query(&params)
        .header("User-Agent", "MCP-Server/1.0")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| anyhow!("Failed to send request to SearXNG: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow!("SearXNG request failed with status {}: {}", status, error_text));
    }
    
    // Parse response
    let searxng_response: SearxngResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse SearXNG response: {}", e))?;
    
    info!("SearXNG returned {} results", searxng_response.results.len());
    
    // Convert to our format
    let results: Vec<SearchResult> = searxng_response
        .results
        .into_iter()
        .map(|result| SearchResult {
            url: result.url,
            title: result.title,
            content: result.content,
            engine: Some(result.engine),
            score: result.score,
        })
        .collect();
    
    debug!("Converted {} results", results.len());
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_search_web() {
        // This test requires a running SearXNG instance
        // Skip in CI/CD environments
        if std::env::var("CI").is_ok() {
            return;
        }
        
        let state = Arc::new(AppState {
            searxng_url: "http://localhost:8888".to_string(),
            http_client: reqwest::Client::new(),
        });
        
        let results = search_web(&state, "rust programming language").await;
        
        match results {
            Ok(results) => {
                assert!(!results.is_empty(), "Should return some results");
                for result in &results {
                    assert!(!result.url.is_empty(), "URL should not be empty");
                    assert!(!result.title.is_empty(), "Title should not be empty");
                }
            }
            Err(e) => {
                // If SearXNG is not running, this is expected
                println!("Search test failed (expected if SearXNG not running): {}", e);
            }
        }
    }
}