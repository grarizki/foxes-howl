# gh-opportunities

Find open source contribution opportunities on GitHub. Scan repos for good first issues, stale PRs, README gaps, and code quality signals. Interactive TUI dashboard included.

## Features

- **Good First Issue Scanner** — finds issues labeled `good first issue`, `help wanted`, `beginner`, `easy`, `starter` and scores them
- **Stale Issue/PR Detector** — identifies issues and PRs with no activity for N days
- **README Analyzer** — checks for CONTRIBUTING.md, CODE_OF_CONDUCT, LICENSE, issue templates, PR templates, build instructions
- **Code Quality Signals** — counts TODO/FIXME/HACK comments, checks for CI config, lint config, test directories
- **Composite Scoring** — ranks repos by contribution opportunity potential
- **Interactive TUI** — terminal dashboard with keyboard navigation, filtering, and detail views
- **SQLite Cache** — local caching to avoid redundant API calls
- **JSON Export** — machine-readable output for scripting
- **AI Analysis** — LLM-powered contribution summaries, difficulty ratings, personalized recommendations (OpenAI + Anthropic)
- **Security Gate** — pre-push checks: CVE scanning, secret detection, license compliance, quality gate
- **HTTP Server** — agent integration via bearer-authenticated REST API
- **OpenAI Tools API** — function definitions for agent-driven contribution discovery
- **Git Hooks** — automatic pre-push security gate installation

## Install

```bash
# from source
git clone https://github.com/grarizki/foxes-howl.git
cd foxes-howl
cargo install --path .

# or build locally
cargo build --release
```

## Setup

Set a GitHub token for higher rate limits (60 req/hr without, 5000 with):

```bash
export GITHUB_TOKEN="ghp_your_token_here"
```

Create default config:

```bash
gh-opportunities init
```

This creates `~/.config/gh-opportunities/config.toml`:

```toml
[scoring]
stale_days = 30
good_first_weight = 0.3
stale_weight = 0.2
readme_weight = 0.2
code_quality_weight = 0.3

[display]
max_results = 25

[ai]
provider = "openai"
model = "gpt-4o"
api_key_env = "OPENAI_API_KEY"
max_tokens = 2048

[ai.profile]
skills = []
experience = "intermediate"
hours_per_week = 4
interests = []

[serve]
token_env = "GH_OPP_TOKEN"

[security]
deny_config_path = ""
secret_patterns = []
```

## Usage

### Scan for Good First Issues

```bash
# basic scan
gh-opportunities scan rust-lang/rust

# limit results
gh-opportunities scan tokio-rs/tokio --limit 10

# JSON output for scripting
gh-opportunities scan denoland/deno --json

# skip cache
gh-opportunities scan facebook/react --no-cache
```

### Discover Repos with Contribution Opportunities

```bash
# discover repos with good first issues in Rust
gh-opportunities discover --lang rust

# discover repos by topic
gh-opportunities discover --topic web --lang typescript

# discover repos with minimum stars
gh-opportunities discover --lang python --min-stars 500 --limit 5

# JSON output
gh-opportunities discover --lang go --json
```

### Find Stale Issues and PRs

```bash
# default: 30 day threshold
gh-opportunities stale rust-lang/rust

# custom threshold
gh-opportunities stale tokio-rs/tokio --days 14

# JSON output
gh-opportunities stale vercel/next.js --json --limit 50
```

### Analyze README and Community Health

```bash
gh-opportunities readme rust-lang/rust
gh-opportunities readme facebook/react --json
```

Output shows:
```
README Analysis for rust-lang/rust (score: 85%)

┌─────────────────────┬─────────┐
│ Check               │ Status  │
├─────────────────────┼─────────┤
│ README.md           │ OK      │
│ CONTRIBUTING.md     │ OK      │
│ CODE_OF_CONDUCT.md  │ OK      │
│ LICENSE             │ OK      │
│ Issue Template      │ OK      │
│ PR Template         │ OK      │
│ Build Instructions  │ OK      │
└─────────────────────┴─────────┘
```

### Analyze Code Quality

```bash
gh-opportunities quality rust-lang/rust
gh-opportunities quality tokio-rs/tokio --json
```

Output shows:
```
Code Quality Analysis for rust-lang/rust (score: 72%)

┌──────────────┬─────────┐
│ Check        │ Value   │
├──────────────┼─────────┤
│ TODO count   │ 234     │
│ FIXME count  │ 45      │
│ HACK count   │ 12      │
│ CI Config    │ OK      │
│ Lint Config  │ OK      │
│ Test Dir     │ OK      │
└──────────────┴─────────┘
```

### Interactive TUI

```bash
# single repo
gh-opportunities tui rust-lang/rust

# multiple repos
gh-opportunities tui rust-lang/rust tokio-rs/tokio denoland/deno
```

#### TUI Keybindings

| Key | Action |
|-----|--------|
| `j` / `Down` | Next item |
| `k` / `Up` | Previous item |
| `Tab` | Next screen |
| `Shift+Tab` | Previous screen |
| `d` | Dashboard |
| `i` | Issues |
| `r` | Repos |
| `Enter` | View detail |
| `/` | Start filter |
| `c` | Clear filter |
| `q` / `Esc` | Quit |

#### TUI Screens

- **Dashboard** — overview: total opportunities, stale items, top matches, repo health
- **Issues** — filterable table of contribution candidates with preview pane
- **Repos** — repo health scores with composite ranking
- **Detail** — full context for selected issue (metadata, labels, description)

### AI-Powered Analysis

Requires an API key. Set in config or environment:

```bash
export OPENAI_API_KEY="sk-..."
# or
export ANTHROPIC_API_KEY="sk-ant-..."
```

```bash
# AI summary of contribution landscape
gh-opportunities ai analyze rust-lang/rust

# Personalized recommendations based on your skills
gh-opportunities ai recommend tokio-rs/tokio --skills "rust,async,web" --hours 10

# Rate issue difficulty
gh-opportunities ai difficulty denoland/deno

# Skip confirmation prompt
gh-opportunities ai analyze rust-lang/rust --yes
```

All AI commands output structured JSON. Token cost estimate shown before each call.

### OpenAI Tools API

```bash
# Output tool definitions for OpenAI function calling
gh-opportunities tools

# Execute a tool call (for agents)
gh-opportunities call scan_issues --args '{"repo":"rust-lang/rust","limit":10}'
```

### HTTP Server

```bash
# Set a bearer token
export GH_OPP_TOKEN="your-secret-token"

# Start server (default port 3737, binds to 127.0.0.1 only)
gh-opportunities serve --port 3737
```

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/health` | No | Health check |
| `GET` | `/tools` | Yes | OpenAI tool definitions |
| `POST` | `/call` | Yes | Execute a tool call |
| `POST` | `/ai/analyze` | Yes | AI analysis |
| `POST` | `/ai/recommend` | Yes | AI recommendations |
| `POST` | `/ai/difficulty` | Yes | Issue difficulty ratings |
| `POST` | `/security` | Yes | Run security checks |
| `GET` | `/profile` | Yes | User profile from config |

### Security Gate

```bash
# Run all security checks
gh-opportunities security

# Run specific check
gh-opportunities security --check audit
gh-opportunities security --check secrets
gh-opportunities security --check quality
gh-opportunities security --check license

# JSON output
gh-opportunities security --json

# Auto-fix (currently: cargo fmt only)
gh-opportunities security --fix
```

Checks:
- **cargo-audit** — CVE scanning for Rust dependencies
- **secrets** — regex-based detection of API keys, tokens, passwords, private keys
- **license** — license compliance via cargo-deny or fallback metadata check
- **quality** — cargo fmt, clippy, and test gate

### Git Hooks

```bash
# Install pre-push hook (runs `gh-opportunities security` before every push)
gh-opportunities hooks install

# Remove pre-push hook
gh-opportunities hooks remove
```

The hook blocks pushes if any security check fails.

## Scoring System

### Issue Score (0.0 - 1.0)

| Signal | Weight | Condition |
|--------|--------|-----------|
| Label match | 0.5 | Has `good first issue`, `help wanted`, etc. |
| Body quality | 0.3 | Description > 50 chars |
| Unassigned | 0.2 | No one assigned |

### Composite Repo Score (0.0 - 1.0)

Higher score = more contribution opportunity:

| Factor | Default Weight | Meaning |
|--------|---------------|---------|
| Good first issues | 0.3 | Average issue score |
| Stale items | 0.2 | Ratio of stale issues/PRs |
| README gaps | 0.2 | Missing community health files |
| Code quality | 0.3 | TODO/FIXME count, missing CI/tests/lint |

## Architecture

```
src/
├── main.rs              # CLI dispatch, table rendering
├── cli.rs               # clap derive definitions
├── config.rs            # XDG config, TOML parsing
├── db.rs                # SQLite cache (rusqlite)
├── github/
│   ├── mod.rs           # octocrab client init
│   ├── issues.rs        # Issue fetching + scoring
│   └── discover.rs      # Repo discovery
├── analysis/
│   ├── mod.rs
│   ├── stale.rs         # Stale PR/issue detection
│   ├── readme.rs        # README gap analysis
│   ├── code_quality.rs  # TODO/FIXME, CI, tests, lint
│   └── scoring.rs       # Composite scoring
├── ai/
│   ├── mod.rs           # Re-exports
│   ├── provider.rs      # LlmProvider trait + factory
│   ├── openai.rs        # OpenAI client
│   ├── anthropic.rs     # Anthropic client
│   ├── prompts.rs       # Prompt templates (analyze, recommend, difficulty)
│   ├── estimate.rs      # Token count + cost estimation
│   └── tools.rs         # OpenAI function-calling definitions
├── security/
│   ├── mod.rs           # SecurityReport, runner
│   ├── audit.rs         # cargo-audit wrapper
│   ├── secrets.rs       # Regex-based secret scanner
│   ├── license.rs       # License compliance check
│   └── quality.rs       # fmt/clippy/test gate
├── serve/
│   ├── mod.rs           # axum HTTP server + bearer auth
│   └── routes.rs        # Route handlers
├── hooks/
│   └── mod.rs           # Pre-push hook installer
└── tui/
    ├── mod.rs           # Terminal setup, event loop
    ├── app.rs           # App state, navigation
    └── screens/
        ├── dashboard.rs # Summary view
        ├── issues.rs    # Issue table + preview
        ├── repos.rs     # Repo health table
        └── detail.rs    # Issue deep-dive
```

## Tech Stack

| Layer | Crate |
|-------|-------|
| CLI | clap 4 (derive) |
| TUI | ratatui + crossterm |
| GitHub API | octocrab |
| Async | tokio |
| DB | rusqlite (bundled) |
| Errors | thiserror (lib), anyhow (binary) |
| HTTP | reqwest + rustls |
| HTTP Server | axum 0.8 |
| Config | dirs (XDG), serde + toml |
| Regex | regex |

## Compile Optimizations

```toml
# .cargo/config.toml (already included)
[profile.dev]
opt-level = 0
debug = true
incremental = true

[profile.release]
strip = true
opt-level = 3
lto = "thin"
codegen-units = 1
```

For faster builds, add to your shell profile:

```bash
export RUSTC_WRAPPER=~/.cargo/bin/sccache  # if sccache installed
export CARGO_TARGET_DIR=~/.cargo/target     # shared target dir
```

## Testing

```bash
# run all tests
cargo test

# run specific module
cargo test analysis::stale

# run with output
cargo test -- --nocapture
```

109 unit tests covering:
- Issue scoring (label matching, body quality, assignment)
- Stale severity calculation (threshold, linear decay, cap)
- README analysis (build instructions, community files, broken links)
- Code quality scoring (penalties, caps, CI/lint/test checks)
- Composite scoring (weight application, edge cases)
- SQLite cache (store, load, clear, replace, schema)
- TUI app state (navigation, filtering, input modes)
- CLI parsing (repo format validation)
- Discover scoring (star bonus calculation)
- AI provider factory (openai, anthropic, missing key, unknown provider)
- AI prompt generation (analyze, recommend, difficulty templates)
- Token estimation (pricing, clamping, formatting)
- OpenAI tool definitions (schema shape, required fields, tool names)
- Security report (pass, fail, unavailable tools)
- Secret detection (AWS keys, GitHub tokens, private keys, generic secrets)
- Audit parsing (cargo-audit JSON, vulnerabilities)
- License checking (deny JSON, allowlist, copyleft detection)
- Serve auth (valid token, invalid token, missing header)
- Hooks (git dir detection, hook content format)
- Config parsing (AI, serve, security sections)

## Security Audit

Check dependencies for vulnerabilities before building:

```bash
# install cargo-audit (if not installed)
brew install cargo-audit

# run security audit
cargo audit
```

Current status: 2 allowed warnings (unmaintained crate `paste`, unsound crate `lru` — both transitive dependencies, not direct).

## Dependencies

```toml
clap = "4"              # CLI parsing
octocrab = "0.44"       # GitHub API client
tokio = "1"             # Async runtime
ratatui = "0.29"        # Terminal UI
crossterm = "0.28"      # Terminal backend
rusqlite = "0.35"       # SQLite (bundled)
reqwest = "0.12"        # HTTP client
axum = "0.8"            # HTTP server
serde = "1"             # Serialization
serde_json = "1"        # JSON
chrono = "0.4"          # Date/time
comfy-table = "7"       # CLI tables
toml = "0.8"            # Config parsing
dirs = "6"              # XDG directories
regex = "1"             # Pattern matching
tracing = "0.1"         # Logging
thiserror = "2"         # Typed errors
anyhow = "1"            # Error context
```

## License

MIT
