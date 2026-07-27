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
gh-opp init
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
```

## Usage

### Scan for Good First Issues

```bash
# basic scan
gh-opp scan rust-lang/rust

# limit results
gh-opp scan tokio-rs/tokio --limit 10

# JSON output for scripting
gh-opp scan denoland/deno --json

# skip cache
gh-opp scan facebook/react --no-cache
```

### Find Stale Issues and PRs

```bash
# default: 30 day threshold
gh-opp stale rust-lang/rust

# custom threshold
gh-opp stale tokio-rs/tokio --days 14

# JSON output
gh-opp stale vercel/next.js --json --limit 50
```

### Analyze README and Community Health

```bash
gh-opp readme rust-lang/rust
gh-opp readme facebook/react --json
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
gh-opp quality rust-lang/rust
gh-opp quality tokio-rs/tokio --json
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
gh-opp tui rust-lang/rust

# multiple repos
gh-opp tui rust-lang/rust tokio-rs/tokio denoland/deno
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
│   └── issues.rs        # Issue fetching + scoring
├── analysis/
│   ├── mod.rs
│   ├── stale.rs         # Stale PR/issue detection
│   ├── readme.rs        # README gap analysis
│   ├── code_quality.rs  # TODO/FIXME, CI, tests, lint
│   └── scoring.rs       # Composite scoring
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
| Config | dirs (XDG), serde + toml |

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

48 unit tests covering:
- Issue scoring (label matching, body quality, assignment)
- Stale severity calculation (threshold, linear decay, cap)
- README analysis (build instructions, community files, broken links)
- Code quality scoring (penalties, caps, CI/lint/test checks)
- Composite scoring (weight application, edge cases)
- SQLite cache (store, load, clear, replace, schema)
- TUI app state (navigation, filtering, input modes)
- CLI parsing (repo format validation)

## Dependencies

```toml
clap = "4"              # CLI parsing
octocrab = "0.44"       # GitHub API client
tokio = "1"             # Async runtime
ratatui = "0.29"        # Terminal UI
crossterm = "0.28"      # Terminal backend
rusqlite = "0.35"       # SQLite (bundled)
reqwest = "0.12"        # HTTP client
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
