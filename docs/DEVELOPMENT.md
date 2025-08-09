# Development Guide for Search-Scrape

## Quick Start

### Option 1: Docker Services (Recommended for Production)
1. **Build and start all services:**
   ```bash
   docker-compose up --build -d
   ```

### Option 2: Native Development (Recommended for Development)
1. **Start SearXNG dependency:**
   ```bash
   docker-compose up -d searxng redis
   ```

2. **Build and run native Rust services:**
   ```bash
   cd mcp-server
   
   # Build release binary
   cargo build --release
   
   # Run MCP stdio server (for MCP integration)
   SEARXNG_URL=http://localhost:8888 ./target/release/mcp-stdio
   
   # OR run HTTP server (for direct API access)
   SEARXNG_URL=http://localhost:8888 cargo run --release --bin mcp-server
   ```

3. **Test the API:**
   ```bash
   curl -X POST "http://localhost:5000/search" \
     -H "Content-Type: application/json" \
     -d '{"query": "test search"}'
   ```

## Service Endpoints

- **SearXNG**: http://localhost:8888
- **MCP HTTP Server**: http://localhost:5000 (when running HTTP mode)
- **MCP stdio Server**: stdio interface (when running MCP mode)

## Debugging in Cursor IDE

### 1. Rust MCP Server Debugging

#### Setup VS Code/Cursor Debug Configuration

Create `.vscode/launch.json`:
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug MCP HTTP Server",
            "cargo": {
                "args": ["build", "--bin=mcp-server"],
                "filter": {
                    "name": "mcp-server",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}/mcp-server",
            "env": {
                "SEARXNG_URL": "http://localhost:8888",
                "RUST_LOG": "debug"
            }
        },
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug MCP stdio Server",
            "cargo": {
                "args": ["build", "--bin=mcp-stdio"],
                "filter": {
                    "name": "mcp-stdio",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}/mcp-server",
            "env": {
                "SEARXNG_URL": "http://localhost:8888",
                "RUST_LOG": "debug"
            }
        }
    ]
}
```

#### Local Development (without Docker)

1. **Start dependencies:**
   ```bash
   # Start only SearXNG and Redis
   docker-compose up -d searxng redis
   ```

2. **Run MCP server locally:**
   ```bash
   cd mcp-server
   
   # For HTTP server debugging
   SEARXNG_URL=http://localhost:8888 cargo run --bin mcp-server
   
   # For stdio server debugging  
   SEARXNG_URL=http://localhost:8888 cargo run --bin mcp-stdio
   ```

3. **Set breakpoints** in Cursor and use F5 to start debugging

### 2. SearXNG Configuration

#### Modify Search Engines

Edit `searxng/settings.yml` to add/remove search engines:
```yaml
engines:
  - name: duckduckgo
    engine: duckduckgo
    shortcut: ddg
    disabled: false
```

#### Enable Debug Mode

Add to `searxng/settings.yml`:
```yaml
general:
  debug: true
  instance_name: "RAG Pipeline SearXNG"
```

## Testing Individual Components

### 1. Test SearXNG
```bash
curl "http://localhost:8888/search?q=test&format=json"
```

### 2. Test MCP Server (HTTP Mode)
```bash
# Health check
curl http://localhost:5000/health

# List MCP tools
curl http://localhost:5000/mcp/tools

# Test MCP tools via HTTP
curl -X POST "http://localhost:5000/mcp/call" \
  -H "Content-Type: application/json" \
  -d '{"name": "search_web", "arguments": {"query": "rust programming"}}'

curl -X POST "http://localhost:5000/mcp/call" \
  -H "Content-Type: application/json" \
  -d '{"name": "scrape_url", "arguments": {"url": "https://www.rust-lang.org/"}}'

# Direct HTTP endpoints (legacy)
curl -X POST "http://localhost:5000/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "test search"}'

curl -X POST "http://localhost:5000/scrape" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'
```

### 3. Test MCP Server (stdio Mode)
The stdio server is designed to be used by MCP clients through the stdio interface. For manual testing, you can use JSON-RPC over stdin/stdout:

```bash
cd mcp-server
echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0.0"}}}' | SEARXNG_URL=http://localhost:8888 ./target/release/mcp-stdio
```

## Troubleshooting

### Common Issues

1. **Port conflicts**: Check if ports 5000, 8888 are available
2. **Docker build fails**: Ensure Docker has enough memory allocated
3. **SearXNG not responding**: Check Redis container is running
4. **MCP stdio connection issues**: Ensure `SEARXNG_URL` environment variable is set
5. **Compilation errors**: Run `cargo check` to identify Rust compilation issues

### Logs

```bash
# View all service logs (Docker mode)
docker-compose logs -f

# View specific service logs (Docker mode)
docker-compose logs -f searxng
docker-compose logs -f redis

# Native Rust server logs
RUST_LOG=info SEARXNG_URL=http://localhost:8888 ./target/release/mcp-stdio
RUST_LOG=debug SEARXNG_URL=http://localhost:8888 cargo run --bin mcp-server
```

### Performance Monitoring

```bash
# Monitor Docker resource usage
docker stats

# Check Docker service health
docker-compose ps

# Check native processes
ps aux | grep -E "(mcp-server|mcp-stdio)"
```

### Build Issues

```bash
# Check for compilation warnings/errors
cd mcp-server
cargo check

# Clean build
cargo clean
cargo build --release

# Update dependencies
cargo update
```

## Development Workflow

### Docker Development
1. **Make changes** to source code
2. **Rebuild specific service**:
   ```bash
   docker-compose up --build mcp-server
   ```
3. **Test changes** using the test script or manual API calls

### Native Development (Recommended)
1. **Make changes** to source code
2. **Check compilation**:
   ```bash
   cd mcp-server
   cargo check
   ```
3. **Build and test**:
   ```bash
   cargo build --release
   SEARXNG_URL=http://localhost:8888 ./target/release/mcp-stdio
   ```
4. **Debug** using Cursor IDE with appropriate configuration

## Integration with Cursor IDE

### Recommended Extensions

- **Rust**: rust-analyzer
- **Docker**: Docker extension
- **YAML**: YAML extension
- **JSON**: JSON Language Features

### Workspace Settings

Create `.vscode/settings.json`:
```json
{
    "rust-analyzer.cargo.target": "x86_64-unknown-linux-gnu",
    "rust-analyzer.check.command": "check",
    "rust-analyzer.cargo.features": "all",
    "docker.defaultRegistryPath": "localhost:5000"
}
```

### Tasks Configuration

Note: Workspace already includes `.vscode/tasks.json` with tasks to build and run services.

## MCP Integration

### MCP Configuration

The project provides MCP tools through the native Rust stdio server. Add this configuration to your MCP client:

```json
{
  "mcpServers": {
    "search-scrape": {
      "command": "/path/to/your/project/mcp-server/target/release/mcp-stdio",
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

1. **search_web**: Search the web using SearXNG federated search engine
   - Parameter: `query` (string) - The search query to execute
   - Returns: List of search results with titles, URLs, and snippets

2. **scrape_url**: Scrape content from URLs using the Rust-native scraper
   - Parameter: `url` (string) - The URL to scrape content from
   - Returns: Cleaned text content, metadata, headings, links, and structured data

### Architecture Overview

```
MCP Client (Trae IDE, etc.)
    ↓ stdio
Rust MCP stdio server (mcp-stdio)
    ↓ internal calls
┌─────────────────┬──────────────────────┐
│   SearXNG       │   Rust-native Scraper│
│ (localhost:8888)│ (in-process)         │
│ Docker container│ Native Rust code     │
└─────────────────┴──────────────────────┘
```

This setup provides a complete development environment for debugging and testing the SearXNG and Rust scraper components in Cursor IDE. The native Rust implementation offers better performance and easier debugging compared to the previous Docker-only approach.