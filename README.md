# Search-Scrape

A **100% free** search and scraping service using SearXNG for federated search, Rust-native web scraping, and a native Rust MCP (Model Context Protocol) server for AI tool integration. **No API keys required** - our goal is to make MCP tools for web search and scraping **free forever**.

## 💰 100% Free - No API Keys Required

**Search-Scrape is completely free and always will be!**

- ✅ **No API keys needed** - Unlike other web scraping and search services
- ✅ **No usage limits** - Search and scrape as much as you need
- ✅ **No subscription fees** - Completely open source and self-hosted
- ✅ **Privacy-focused** - All data stays on your infrastructure
- ✅ **Extensible** - Add your own search engines if you want premium sources

**Our Mission**: Provide free web search and scraping MCP tools forever, making AI assistants more capable without the cost barriers.

## 🏗️ Architecture

- **SearXNG**: Federated search engine aggregating results from multiple sources (DuckDuckGo, Google, Bing, etc.) - **All free sources by default**
- **Rust Scraper**: High-performance native web scraping with content extraction and cleanup
- **Native Rust MCP Server**: Direct MCP protocol implementation exposing `search_web` and `scrape_url` as function tools
- **Docker Compose**: Containerized deployment for easy setup and scaling
- **Trae IDE Integration**: Complete MCP tool integration for AI assistants

## 🚀 Quick Start

### For HTTP API Usage
```bash
# Start all services (detached)
docker-compose up --build -d

# Test the API endpoints
curl -X POST "http://localhost:5000/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "test search"}'
```

### For MCP Tool Integration (Trae IDE)
```bash
# Build the native MCP server (no Docker needed for MCP)
cd mcp-server
cargo build --release

# Configure in your MCP client (see MCP Integration section below)
```

## 🤔 Docker vs Native: When to Use What?

### Docker Usage (SearXNG only)
- **What**: SearXNG search engine service
- **Why Docker**: Isolated environment, easy deployment, web service
- **Command**: `docker-compose up searxng -d`

### Native Usage (MCP Server)
- **What**: MCP server for AI assistant integration
- **Why Native**: Direct stdio communication, no network overhead, better performance
- **Command**: `cargo build --release` then configure in AI assistant

**Key Point**: You need both! Docker for SearXNG (search backend) + Native binary for MCP (AI integration).

## 🌐 Service Endpoints

- **SearXNG**: http://localhost:8888 (federated search via Docker port 8888 → 8080)
- **MCP Server**: http://localhost:5000 (function tools & chat pipeline)

## 📡 API Usage

### Search Web
```bash
curl -X POST "http://localhost:5000/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "machine learning trends 2024", "limit": 5}'
```

### Scrape URL
```bash
curl -X POST "http://localhost:5000/scrape" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'
```

### Chat Pipeline (Search + Scrape + Summarize)
```bash
curl -X POST "http://localhost:5000/chat" \
  -H "Content-Type: application/json" \
  -d '{"query": "latest AI developments", "max_urls": 3}'
```

### List Available Tools
```bash
curl http://localhost:5000/mcp/tools
```

## 🔌 MCP Integration (Vscode, Cursor ,Trae)

The project now includes a native Rust MCP server that can be integrated directly with AI assistants like Trae IDE.

### Configuration

Add this to your MCP client configuration:

```json
{
  "mcpServers": {
    "search-scrape": {
      "command": "/Users/mcp-server/target/release/mcp-stdio",
      "args": [],
      "env": {
        "SEARXNG_URL": "http://localhost:8888"
      },
      "description": "Search the web using SearXNG and scrape content using a Rust-native scraper. Provides 'search_web' for federated search and 'scrape_url' for extracting clean content, metadata, headings, links, and structured data."
    }
  }
}
```

### Available MCP Tools

**Both tools are 100% free with no API keys required!**

- **`search_web`**: Federated web search via SearXNG (free search engines)
  - Input: `{"query": "your search terms"}`
  - Returns: List of search results with titles, URLs, and snippets
  - **No cost**: Unlike Google Custom Search API, Bing Search API, or other paid services

- **`scrape_url`**: Extract content from web pages (native Rust scraper)
  - Input: `{"url": "https://example.com"}`
  - Returns: Clean text content, metadata, headings, links, and images
  - **No cost**: Unlike ScrapingBee, Apify, or other paid scraping services

### 📸 MCP Tools in Action

Here are screenshots showing the MCP tools working in Trae IDE:

#### Search Web Tool
![Search Web Tool Screenshot](docs/Screenshot%202025-08-09%20at%2023.31.06.png)

#### Scrape URL Tool  
![Scrape URL Tool Screenshot](docs/Screenshot%202025-08-09%20at%2023.31.43.png)

The screenshots demonstrate:
- **Real-time search results** from federated search engines
- **Clean content extraction** with metadata, headings, and structured data
- **Seamless integration** with AI assistants through MCP protocol
- **Rich formatting** of scraped content for easy consumption

### Prerequisites for MCP Integration

1. **Start SearXNG** (Docker): `docker-compose up searxng -d`
2. **Build the MCP server** (Native): `cd mcp-server && cargo build --release`
3. **Configure environment**: Set `SEARXNG_URL=http://localhost:8888`

**Note**: Only SearXNG runs in Docker. The MCP server runs natively for direct AI assistant integration.

## 🛠️ Development

- **Trae IDE**: Full MCP tool integration for AI assistants
- **Local Development**: Run services individually for debugging
- **Hot Reload**: Automatic rebuilds on code changes with `cargo watch`
- **Comprehensive Logging**: Structured logs for all components
- **Test Suite**: Automated pipeline validation

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for detailed debugging instructions.

## 📁 Project Structure

```
├── docker-compose.yml          # Service orchestration (SearXNG only)
├── .vscode/                    # Cursor IDE configuration
│   ├── launch.json             # Debug configurations
│   ├── settings.json           # Workspace settings
│   └── tasks.json              # Build & test tasks
├── searxng/
│   ├── settings.yml            # SearXNG configuration
│   └── uwsgi.ini               # SearXNG uWSGI config
├── mcp-server/
│   ├── Cargo.toml              # Rust dependencies
│   ├── Dockerfile              # Rust container
│   ├── src/
│   │   ├── bin/                # Binary targets
│   │   │   ├── mcp-server.rs   # HTTP API server
│   │   │   └── mcp-stdio.rs    # MCP stdio server
│   │   ├── lib.rs              # Library exports
│   │   ├── main.rs             # HTTP server main
│   │   ├── stdio_service.rs    # Native MCP implementation
│   │   ├── search.rs           # SearXNG integration
│   │   ├── scrape.rs           # Rust-native scraper
│   │   ├── rust_scraper.rs     # Scraping engine
│   │   ├── mcp.rs              # HTTP MCP endpoints
│   │   └── types.rs            # Shared data types
│   └── target/
│       ├── debug/              # Debug builds
│       └── release/            # Release builds (for MCP)
├── docs/
│   └── DEVELOPMENT.md          # Development guide
└── README.md
```

## ✨ Features

- **💰 100% Free Forever**: No API keys, no subscriptions, no usage limits - unlike paid alternatives
- **🔍 Federated Search**: Aggregate results from multiple free search engines (DuckDuckGo, Google, Bing, Startpage)
- **🕷️ Smart Scraping**: Extract clean content with metadata, headings, links, and images - no scraping API costs
- **🔧 Native MCP Tools**: Direct MCP protocol implementation for AI assistant integration
- **🛡️ Error Handling**: Robust fallbacks and retry mechanisms
- **🐳 Containerized**: Easy deployment and scaling with Docker Compose
- **🔒 Privacy-First**: All data processing happens locally - no external API calls with your data
- **🐛 Development Ready**: Full Trae IDE integration with MCP tools
- **📊 Monitoring**: Health checks and comprehensive logging
- **🚀 Performance**: Async Rust backend with zero-copy parsing and connection pooling
- **🔄 Dual Interface**: Both HTTP API and native MCP stdio protocols
- **⚡ Zero Dependencies**: Pure Rust implementation without Node.js wrapper
- **🌍 Extensible**: Add premium search engines if you choose to pay for enhanced results

## Architecture

### HTTP API Mode
```
User Query → HTTP Server → SearXNG (search) → Rust Scraper (scrape) → JSON Response
```

### MCP Tool Mode
```
AI Assistant → MCP stdio → SearXNG (search) → Rust Scraper (scrape) → Structured Response
```