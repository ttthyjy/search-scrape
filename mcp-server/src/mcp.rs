use crate::types::*;
use crate::{search, scrape, AppState};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, error};

#[derive(Debug, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpToolsResponse {
    pub tools: Vec<McpTool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpCallRequest {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpCallResponse {
    pub content: Vec<McpContent>,
    pub is_error: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

pub async fn list_tools() -> Json<McpToolsResponse> {
    let tools = vec![
        McpTool {
            name: "search_web".to_string(),
            description: "Search the web using SearXNG federated search engine. Returns a list of relevant URLs with titles and snippets.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query to execute"
                    }
                },
                "required": ["query"]
            }),
        },
        McpTool {
            name: "scrape_url".to_string(),
            description: "Scrape content from a specific URL using a Rust-native scraper. Returns cleaned text content, metadata, and structured data.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to scrape content from"
                    }
                },
                "required": ["url"]
            }),
        },
    ];
    
    Json(McpToolsResponse { tools })
}

pub async fn call_tool(
    State(state): State<Arc<AppState>>,
    Json(request): Json<McpCallRequest>,
) -> Result<Json<McpCallResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("MCP tool call: {} with args: {:?}", request.name, request.arguments);
    
    match request.name.as_str() {
        "search_web" => {
            // Extract query from arguments
            let query = request.arguments
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "Missing required parameter: query".to_string(),
                        }),
                    )
                })?;
            
            // Perform search
            match search::search_web(&state, query).await {
                Ok(results) => {
                    let content_text = if results.is_empty() {
                        format!("No search results found for query: {}", query)
                    } else {
                        let mut text = format!("Found {} search results for '{}':\n\n", results.len(), query);
                        for (i, result) in results.iter().enumerate() {
                            text.push_str(&format!(
                                "{}. **{}**\n   URL: {}\n   Snippet: {}\n\n",
                                i + 1,
                                result.title,
                                result.url,
                                result.content.chars().take(200).collect::<String>()
                            ));
                        }
                        text
                    };
                    
                    Ok(Json(McpCallResponse {
                        content: vec![McpContent {
                            content_type: "text".to_string(),
                            text: content_text,
                        }],
                        is_error: false,
                    }))
                }
                Err(e) => {
                    error!("Search tool error: {}", e);
                    Ok(Json(McpCallResponse {
                        content: vec![McpContent {
                            content_type: "text".to_string(),
                            text: format!("Search failed: {}", e),
                        }],
                        is_error: true,
                    }))
                }
            }
        }
        "scrape_url" => {
            // Extract URL from arguments
            let url = request.arguments
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "Missing required parameter: url".to_string(),
                        }),
                    )
                })?;
            
            // Perform scraping - only Rust-native path
            match scrape::scrape_url(&state, url).await {
                Ok(content) => {
                    let content_text = format!(
                        "**{}**\n\nURL: {}\nWord Count: {}\nLanguage: {}\n\n**Content:**\n{}\n\n**Metadata:**\n- Description: {}\n- Keywords: {}\n\n**Headings:**\n{}\n\n**Links Found:** {}\n**Images Found:** {}",
                        content.title,
                        content.url,
                        content.word_count,
                        content.language,
                        content.clean_content.chars().take(2000).collect::<String>(),
                        content.meta_description,
                        content.meta_keywords,
                        content.headings.iter()
                            .map(|h| format!("- {} {}", h.level.to_uppercase(), h.text))
                            .collect::<Vec<_>>()
                            .join("\n"),
                        content.links.len(),
                        content.images.len()
                    );
                    
                    Ok(Json(McpCallResponse {
                        content: vec![McpContent {
                            content_type: "text".to_string(),
                            text: content_text,
                        }],
                        is_error: false,
                    }))
                }
                Err(e) => {
                    error!("Scrape tool error: {}", e);
                    Ok(Json(McpCallResponse {
                        content: vec![McpContent {
                            content_type: "text".to_string(),
                            text: format!("Scraping failed: {}", e),
                        }],
                        is_error: true,
                    }))
                }
            }
        }
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Unknown tool: {}", request.name),
            }),
        )),
    }
}