use crate::types::*;
use crate::AppState;
use anyhow::{anyhow, Result};
use std::sync::Arc;
use tracing::info;
use select::predicate::Predicate;
use crate::rust_scraper::RustScraper;

pub async fn scrape_url(_state: &Arc<AppState>, url: &str) -> Result<ScrapeResponse> {
    info!("Scraping URL: {}", url);
    
    // Validate URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(anyhow!("Invalid URL: must start with http:// or https://"));
    }

    // Only use Rust-native scraper
    let rust_scraper = RustScraper::new();
    let result = rust_scraper.scrape_url(url).await?;
    info!("Rust-native scraper succeeded for {}", url);
    Ok(result)
}

// Fallback scraper using direct HTTP request (legacy simple mode) -- optional; keeping for troubleshooting
pub async fn scrape_url_fallback(state: &Arc<AppState>, url: &str) -> Result<ScrapeResponse> {
    info!("Using fallback scraper for: {}", url);
    
    // Make direct HTTP request
    let response = state
        .http_client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (compatible; MCP-Server/1.0)")
        .send()
        .await
        .map_err(|e| anyhow!("Failed to fetch URL: {}", e))?;
    
    let status_code = response.status().as_u16();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("text/html")
        .to_string();
    
    let html = response
        .text()
        .await
        .map_err(|e| anyhow!("Failed to read response body: {}", e))?;
    
    let document = select::document::Document::from(html.as_str());
    
    let title = document
        .find(select::predicate::Name("title"))
        .next()
        .map(|n| n.text())
        .unwrap_or_else(|| "No Title".to_string());
    
    let meta_description = document
        .find(select::predicate::Attr("name", "description"))
        .next()
        .and_then(|n| n.attr("content"))
        .unwrap_or("")
        .to_string();
    
    let meta_keywords = document
        .find(select::predicate::Attr("name", "keywords"))
        .next()
        .and_then(|n| n.attr("content"))
        .unwrap_or("")
        .to_string();
    
    let body_html = document
        .find(select::predicate::Name("body"))
        .next()
        .map(|n| n.html())
        .unwrap_or_else(|| html.clone());
    
    let clean_content = html2text::from_read(body_html.as_bytes(), 80);
    let word_count = clean_content.split_whitespace().count();
    
    let headings: Vec<Heading> = document
        .find(select::predicate::Name("h1")
            .or(select::predicate::Name("h2"))
            .or(select::predicate::Name("h3"))
            .or(select::predicate::Name("h4"))
            .or(select::predicate::Name("h5"))
            .or(select::predicate::Name("h6")))
        .map(|n| Heading {
            level: n.name().unwrap_or("h1").to_string(),
            text: n.text(),
        })
        .collect();
    
    let links: Vec<Link> = document
        .find(select::predicate::Name("a"))
        .filter_map(|n| {
            n.attr("href").map(|href| Link {
                url: href.to_string(),
                text: n.text(),
            })
        })
        .collect();
    
    let images: Vec<Image> = document
        .find(select::predicate::Name("img"))
        .filter_map(|n| {
            n.attr("src").map(|src| Image {
                src: src.to_string(),
                alt: n.attr("alt").unwrap_or("").to_string(),
                title: n.attr("title").unwrap_or("").to_string(),
            })
        })
        .collect();
    
    let result = ScrapeResponse {
        url: url.to_string(),
        title,
        content: html,
        clean_content,
        meta_description,
        meta_keywords,
        headings,
        links,
        images,
        timestamp: chrono::Utc::now().to_rfc3339(),
        status_code,
        content_type,
        word_count,
        language: "unknown".to_string(),
    };
    
    info!("Fallback scraper extracted {} words", result.word_count);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_scrape_url_fallback() {
        let state = Arc::new(AppState {
            searxng_url: "http://localhost:8888".to_string(),
            http_client: reqwest::Client::new(),
        });
        
        let result = scrape_url_fallback(&state, "https://httpbin.org/html").await;
        
        match result {
            Ok(content) => {
                assert!(!content.title.is_empty(), "Title should not be empty");
                assert!(!content.clean_content.is_empty(), "Content should not be empty");
                assert_eq!(content.status_code, 200, "Status code should be 200");
            }
            Err(e) => {
                println!("Fallback scraper test failed: {}", e);
            }
        }
    }
}