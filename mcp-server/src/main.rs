use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::env;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn, error};

use mcp_server::{search, scrape, types::*, mcp, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Get configuration from environment
    let searxng_url = env::var("SEARXNG_URL")
        .unwrap_or_else(|_| "http://localhost:8888".to_string());
    
    info!("Starting MCP Server");
    info!("SearXNG URL: {}", searxng_url);

    // Create HTTP client
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    // Create application state
    let state = Arc::new(AppState {
        searxng_url,
        http_client,
    });

    // Build router
    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/search", post(search_web_handler))
        .route("/scrape", post(scrape_url_handler))
        .route("/chat", post(chat_handler))
        .route("/mcp/tools", get(mcp::list_tools))
        .route("/mcp/call", post(mcp::call_tool))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await?;
    info!("MCP Server listening on http://0.0.0.0:5000");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "mcp-server",
        "version": "0.1.0"
    }))
}

async fn search_web_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    match search::search_web(&state, &request.query).await {
        Ok(results) => Ok(Json(SearchResponse { results })),
        Err(e) => {
            error!("Search error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            ))
        }
    }
}

async fn scrape_url_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ScrapeRequest>,
) -> Result<Json<ScrapeResponse>, (StatusCode, Json<ErrorResponse>)> {
    match scrape::scrape_url(&state, &request.url).await {
        Ok(content) => Ok(Json(content)),
        Err(e) => {
            error!("Scrape error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            ))
        }
    }
}

async fn chat_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Processing chat request: {}", request.query);
    
    // Step 1: Search for relevant URLs
    let search_results = match search::search_web(&state, &request.query).await {
        Ok(results) => results,
        Err(e) => {
            error!("Search failed: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Search failed: {}", e),
                }),
            ));
        }
    };
    
    info!("Found {} search results", search_results.len());
    
    // Step 2: Scrape top results (limit to 3 for demo)
    let mut scraped_content = Vec::new();
    for result in search_results.iter().take(3) {
        match scrape::scrape_url(&state, &result.url).await {
            Ok(content) => {
                info!("Successfully scraped: {}", result.url);
                scraped_content.push(content);
            }
            Err(e) => {
                warn!("Failed to scrape {}: {}", result.url, e);
            }
        }
    }
    
    // Step 3: Generate response based on scraped content
    let response_text = if scraped_content.is_empty() {
        format!("I found {} search results for '{}', but couldn't scrape any content. Here are the URLs:\n{}", 
            search_results.len(),
            request.query,
            search_results.iter().map(|r| format!("- {} ({})", r.title, r.url)).collect::<Vec<_>>().join("\n")
        )
    } else {
        let content_summary = scraped_content.iter()
            .map(|c| format!("**{}**\n{}\n", c.title, c.clean_content.chars().take(500).collect::<String>()))
            .collect::<Vec<_>>()
            .join("\n---\n");
        
        format!("Based on my search for '{}', I found the following information:\n\n{}", 
            request.query, content_summary)
    };
    
    Ok(Json(ChatResponse {
        response: response_text,
        search_results,
        scraped_content,
    }))
}