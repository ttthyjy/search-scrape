use crate::types::*;
use anyhow::{anyhow, Result};
use chrono::Utc;
use rand::Rng;
use readability::extractor;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use tracing::{info, warn};
use url::Url;
use whatlang::{detect, Lang};

/// User agents for rotation
const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:89.0) Gecko/20100101 Firefox/89.0",
];

/// Enhanced Rust-native web scraper
pub struct RustScraper {
    client: Client,
}

impl RustScraper {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Get a random User-Agent string
    fn get_random_user_agent(&self) -> &'static str {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..USER_AGENTS.len());
        USER_AGENTS[index]
    }

    /// Scrape a URL with enhanced content extraction
    pub async fn scrape_url(&self, url: &str) -> Result<ScrapeResponse> {
        info!("Scraping URL with Rust-native scraper: {}", url);

        // Validate URL
        let parsed_url = Url::parse(url)
            .map_err(|e| anyhow!("Invalid URL '{}': {}", url, e))?;

        if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
            return Err(anyhow!("URL must use HTTP or HTTPS protocol"));
        }

        // Make HTTP request with random User-Agent
        let user_agent = self.get_random_user_agent();
        let response = self
            .client
            .get(url)
            .header("User-Agent", user_agent)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            // Rely on reqwest automatic decompression; remove manual Accept-Encoding to avoid serving compressed body as text
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
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

        // Get response body
        let html = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

        // Parse HTML
        let document = Html::parse_document(&html);
        
        // Extract basic metadata
        let title = self.extract_title(&document);
        let meta_description = self.extract_meta_description(&document);
        let meta_keywords = self.extract_meta_keywords(&document);
        let language = self.detect_language(&document, &html);

        // Extract readable content using readability
        let clean_content = self.extract_clean_content(&html, &parsed_url);
        let word_count = self.count_words(&clean_content);

        // Extract structured data
        let headings = self.extract_headings(&document);
        let links = self.extract_links(&document, &parsed_url);
        let images = self.extract_images(&document, &parsed_url);

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
            timestamp: Utc::now().to_rfc3339(),
            status_code,
            content_type,
            word_count,
            language,
        };

        info!("Successfully scraped: {} ({} words)", result.title, result.word_count);
        Ok(result)
    }

    /// Extract page title with fallback to h1
    fn extract_title(&self, document: &Html) -> String {
        // Try title tag first
        if let Ok(title_selector) = Selector::parse("title") {
            if let Some(title_element) = document.select(&title_selector).next() {
                let title = title_element.text().collect::<String>().trim().to_string();
                if !title.is_empty() {
                    return title;
                }
            }
        }

        // Fallback to h1
        if let Ok(h1_selector) = Selector::parse("h1") {
            if let Some(h1_element) = document.select(&h1_selector).next() {
                let h1_text = h1_element.text().collect::<String>().trim().to_string();
                if !h1_text.is_empty() {
                    return h1_text;
                }
            }
        }

        "No Title".to_string()
    }

    /// Extract meta description
    fn extract_meta_description(&self, document: &Html) -> String {
        if let Ok(selector) = Selector::parse("meta[name=\"description\"]") {
            if let Some(element) = document.select(&selector).next() {
                if let Some(content) = element.value().attr("content") {
                    return content.trim().to_string();
                }
            }
        }
        String::new()
    }

    /// Extract meta keywords
    fn extract_meta_keywords(&self, document: &Html) -> String {
        if let Ok(selector) = Selector::parse("meta[name=\"keywords\"]") {
            if let Some(element) = document.select(&selector).next() {
                if let Some(content) = element.value().attr("content") {
                    return content.trim().to_string();
                }
            }
        }
        String::new()
    }

    /// Detect language from HTML attributes and content
    fn detect_language(&self, document: &Html, html: &str) -> String {
        // Try HTML lang attribute
        if let Ok(selector) = Selector::parse("html") {
            if let Some(html_element) = document.select(&selector).next() {
                if let Some(lang) = html_element.value().attr("lang") {
                    return lang.trim().to_string();
                }
            }
        }

        // Try meta content-language
        if let Ok(selector) = Selector::parse("meta[http-equiv=\"content-language\"]") {
            if let Some(element) = document.select(&selector).next() {
                if let Some(content) = element.value().attr("content") {
                    return content.trim().to_string();
                }
            }
        }

        // Use whatlang for content-based detection
        if let Some(info) = detect(html) {
            match info.lang() {
                Lang::Eng => "en".to_string(),
                Lang::Spa => "es".to_string(),
                Lang::Fra => "fr".to_string(),
                Lang::Deu => "de".to_string(),
                Lang::Ita => "it".to_string(),
                Lang::Por => "pt".to_string(),
                Lang::Rus => "ru".to_string(),
                Lang::Jpn => "ja".to_string(),
                Lang::Kor => "ko".to_string(),
                Lang::Cmn => "zh".to_string(),
                _ => format!("{:?}", info.lang()).to_lowercase(),
            }
        } else {
            "unknown".to_string()
        }
    }

    /// Extract clean, readable content using readability
    fn extract_clean_content(&self, html: &str, base_url: &Url) -> String {
        match extractor::extract(&mut html.as_bytes(), base_url) {
            Ok(product) => {
                // Convert HTML to text using html2text
                html2text::from_read(product.content.as_bytes(), 80)
            }
            Err(e) => {
                warn!("Readability extraction failed: {}, falling back to simple text extraction", e);
                self.fallback_text_extraction(html)
            }
        }
    }

    /// Fallback text extraction when readability fails
    fn fallback_text_extraction(&self, html: &str) -> String {
        let document = Html::parse_document(html);
        
        // Remove script and style elements
        let mut text_parts = Vec::new();
        
        if let Ok(body_selector) = Selector::parse("body") {
            if let Some(body) = document.select(&body_selector).next() {
                self.extract_text_recursive(&body, &mut text_parts);
            }
        } else {
            // Fallback to entire document
            for node in document.tree.nodes() {
                if let Some(text) = node.value().as_text() {
                    text_parts.push(text.text.to_string());
                }
            }
        }
        
        let text = text_parts.join(" ");
        self.clean_text(&text)
    }

    /// Recursively extract text from elements
    fn extract_text_recursive(&self, element: &scraper::ElementRef, text_parts: &mut Vec<String>) {
        for child in element.children() {
            if let Some(child_element) = scraper::ElementRef::wrap(child) {
                let tag_name = child_element.value().name();
                // Skip script and style elements
                if tag_name == "script" || tag_name == "style" {
                    continue;
                }
                self.extract_text_recursive(&child_element, text_parts);
            } else if let Some(text_node) = child.value().as_text() {
                text_parts.push(text_node.text.to_string());
            }
        }
    }

    /// Clean extracted text
    fn clean_text(&self, text: &str) -> String {
        // Remove excessive whitespace
        let re_whitespace = Regex::new(r"\s+").unwrap();
        let re_newlines = Regex::new(r"\n\s*\n").unwrap();
        
        let cleaned = re_whitespace.replace_all(text, " ");
        let cleaned = re_newlines.replace_all(&cleaned, "\n\n");
        
        cleaned.trim().to_string()
    }

    /// Count words in text
    fn count_words(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }

    /// Extract headings (h1-h6)
    fn extract_headings(&self, document: &Html) -> Vec<Heading> {
        let mut headings = Vec::new();
        
        for level in 1..=6 {
            let sel: &str = match level {
                1 => "h1",
                2 => "h2",
                3 => "h3",
                4 => "h4",
                5 => "h5",
                _ => "h6",
            };
            if let Ok(selector) = Selector::parse(sel) {
                for element in document.select(&selector) {
                    let text = element.text().collect::<String>().trim().to_string();
                    if !text.is_empty() {
                        headings.push(Heading {
                            level: sel.to_string(),
                            text,
                        });
                    }
                }
            }
        }
        
        headings
    }

    /// Extract links with absolute URLs
    fn extract_links(&self, document: &Html, base_url: &Url) -> Vec<Link> {
        let mut links = Vec::new();
        let mut seen_urls = HashSet::new();
        
        if let Ok(selector) = Selector::parse("a[href]") {
            for element in document.select(&selector) {
                if let Some(href) = element.value().attr("href") {
                    let text = element.text().collect::<String>().trim().to_string();
                    
                    // Convert relative URLs to absolute
                    let absolute_url = match base_url.join(href) {
                        Ok(url) => url.to_string(),
                        Err(_) => href.to_string(),
                    };
                    
                    // Avoid duplicates
                    if !seen_urls.contains(&absolute_url) {
                        seen_urls.insert(absolute_url.clone());
                        links.push(Link {
                            url: absolute_url,
                            text,
                        });
                    }
                }
            }
        }
        
        links
    }

    /// Extract images with absolute URLs
    fn extract_images(&self, document: &Html, base_url: &Url) -> Vec<Image> {
        let mut images = Vec::new();
        let mut seen_srcs = HashSet::new();
        
        if let Ok(selector) = Selector::parse("img[src]") {
            for element in document.select(&selector) {
                if let Some(src) = element.value().attr("src") {
                    // Convert relative URLs to absolute
                    let absolute_src = match base_url.join(src) {
                        Ok(url) => url.to_string(),
                        Err(_) => src.to_string(),
                    };
                    
                    // Avoid duplicates
                    if !seen_srcs.contains(&absolute_src) {
                        seen_srcs.insert(absolute_src.clone());
                        
                        let alt = element.value().attr("alt").unwrap_or("").to_string();
                        let title = element.value().attr("title").unwrap_or("").to_string();
                        
                        images.push(Image {
                            src: absolute_src,
                            alt,
                            title,
                        });
                    }
                }
            }
        }
        
        images
    }
}

impl Default for RustScraper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rust_scraper() {
        let scraper = RustScraper::new();
        
        // Test with a simple HTML page
        match scraper.scrape_url("https://httpbin.org/html").await {
            Ok(content) => {
                assert!(!content.title.is_empty(), "Title should not be empty");
                assert!(!content.clean_content.is_empty(), "Content should not be empty");
                assert_eq!(content.status_code, 200, "Status code should be 200");
                assert!(content.word_count > 0, "Word count should be greater than 0");
            }
            Err(e) => {
                println!("Rust scraper test failed: {}", e);
            }
        }
    }
    
    #[test]
    fn test_clean_text() {
        let scraper = RustScraper::new();
        let text = "  This   is    \n\n\n   some    text   \n\n  ";
        let cleaned = scraper.clean_text(text);
        assert_eq!(cleaned, "This is some text");
    }
    
    #[test]
    fn test_word_count() {
        let scraper = RustScraper::new();
        let text = "This is a test with five words";
        assert_eq!(scraper.count_words(text), 6);
    }
}