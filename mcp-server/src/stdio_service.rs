use rmcp::{model::*, ServiceExt};
use std::env;
use std::sync::Arc;
use tracing::{error, info};
use std::borrow::Cow;

// Re-export types from our main module
use crate::{search, scrape, AppState};

#[derive(Clone, Debug)]
pub struct McpService {
    pub state: Arc<AppState>,
}

impl McpService {
    pub fn new() -> anyhow::Result<Self> {
        // Initialize tracing
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();

        // Get configuration from environment
        let searxng_url = env::var("SEARXNG_URL")
            .unwrap_or_else(|_| "http://localhost:8888".to_string());
        
        info!("Starting MCP Service");
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

        Ok(Self { state })
    }
}

impl rmcp::ServerHandler for McpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            server_info: Implementation {
                name: "search-scrape".to_string(),
                version: "1.0.0".to_string(),
            },
            instructions: Some(
                "A pure Rust web search and scraping service using SearXNG for federated search and a native Rust scraper for content extraction.".to_string(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _page: Option<PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let tools = vec![
            Tool {
                name: Cow::Borrowed("search_web"),
                description: Some(Cow::Borrowed("Search the web using SearXNG federated search engine. Returns a list of relevant URLs with titles and snippets.")),
                input_schema: match serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query to execute"
                        }
                    },
                    "required": ["query"]
                }) {
                    serde_json::Value::Object(map) => std::sync::Arc::new(map),
                    _ => std::sync::Arc::new(serde_json::Map::new()),
                },
                output_schema: None,
                annotations: None,
            },
            Tool {
                name: Cow::Borrowed("scrape_url"),
                description: Some(Cow::Borrowed("Scrape content from a specific URL using a Rust-native scraper. Returns cleaned text content, metadata, and structured data.")),
                input_schema: match serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to scrape content from"
                        }
                    },
                    "required": ["url"]
                }) {
                    serde_json::Value::Object(map) => std::sync::Arc::new(map),
                    _ => std::sync::Arc::new(serde_json::Map::new()),
                },
                output_schema: None,
                annotations: None,
            },
        ];

        Ok(ListToolsResult {
            tools,
            ..Default::default()
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("MCP tool call: {} with args: {:?}", request.name, request.arguments);
        
        match request.name.as_ref() {
            "search_web" => {
                // Extract query from arguments
                let args = request.arguments.as_ref().ok_or_else(|| ErrorData::new(
                    ErrorCode::INVALID_PARAMS,
                    "Missing required arguments object",
                    None,
                ))?;
                let query = args
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ErrorData::new(
                        ErrorCode::INVALID_PARAMS,
                        "Missing required parameter: query",
                        None,
                    ))?;
                
                // Perform search
                match search::search_web(&self.state, query).await {
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
                        
                        Ok(CallToolResult::success(vec![Content::text(content_text)]))
                    }
                    Err(e) => {
                        error!("Search tool error: {}", e);
                        Ok(CallToolResult::success(vec![Content::text(format!("Search failed: {}", e))]))
                    }
                }
            }
            "scrape_url" => {
                // Extract URL from arguments
                let args = request.arguments.as_ref().ok_or_else(|| ErrorData::new(
                    ErrorCode::INVALID_PARAMS,
                    "Missing required arguments object",
                    None,
                ))?;
                let url = args
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ErrorData::new(
                        ErrorCode::INVALID_PARAMS,
                        "Missing required parameter: url",
                        None,
                    ))?;
                
                // Perform scraping
                match scrape::scrape_url(&self.state, url).await {
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
                        
                        Ok(CallToolResult::success(vec![Content::text(content_text)]))
                    }
                    Err(e) => {
                        error!("Scrape tool error: {}", e);
                        Ok(CallToolResult::success(vec![Content::text(format!("Scraping failed: {}", e))]))
                    }
                }
            }
            _ => Err(ErrorData::new(
                ErrorCode::METHOD_NOT_FOUND,
                format!("Unknown tool: {}", request.name),
                None,
            )),
        }
    }
}

pub async fn run() -> anyhow::Result<()> {
    let service = McpService::new()?;
    // Use the stdio transport from rmcp
    let server = service.serve(rmcp::transport::stdio()).await?;
    info!("MCP stdio server running");
    let _quit_reason = server.waiting().await?;
    Ok(())
}