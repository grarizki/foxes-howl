<div align="center">

# gh-opportunities

**Find your next open-source contribution on GitHub.** Scan repositories for beginner-friendly issues, stale pull requests, README gaps, and code-quality signals — then rank them by how much you'll learn and how big an impact you can make.

Rust · Async (tokio) · Terminal UI (ratatui) · HTTP API (axum) · SQLite cache · AI-powered recommendations

</div>

<p align="center">
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
  <img src="https://img.shields.io/badge/rust-1.88+-orange.svg" alt="MSRV 1.88">
  <img src="https://img.shields.io/badge/CI-precommit%2Faudit%2Fcoverage-brightgreen.svg" alt="CI">
  <img src="https://img.shields.io/badge/tests-109%20passing-brightgreen.svg" alt="Tests">
  <a href="https://github.com/grarizki/foxes-howl"><img src="https://img.shields.io/github/stars/grarizki/foxes-howl?style=social"></a>
</p>

---

## Why this exists

Looking for good open-source contributions is slow. You open a repo, hunt for `good first issue` labels, guess whether issues are stale, and ignore more than you keep. **gh-opportunities automates that research** and turns it into a ranked, scored list you can act on in minutes — from the terminal, from a script, or from an AI agent.

It goes beyond "find issues labeled beginner." It scores repos on four signals:

- **Good-first-issue quality** — labeled, well-described, unassigned
- **Staleness** — issues and PRs waiting too long for attention
- **README & community health** — CONTRIBUTING.md, CoC, LICENSE, templates
- **Code quality** — TODO/FIXME density, CI, lint config, test coverage

The composite score tells you *where you'll make the most difference* — not just *where something is labeled easy*.

---

## Features

- **🔍 Repo scanner** — fetch and score good-first issues for any `owner/repo`
- **🗺️ Repo discovery** — find high-opportunity repos by language and topic
- **⏳ Stale detector** — surface issues and PRs idle for N days (configurable)
- **📖 README/community audit** — check the 7 community-health files every project should have
- **🧹 Code-quality analysis** — TODO/FIXME/HACK counts, CI, lint, test signals
- **🧮 Composite scoring** — transparent, weighted repo ranking
- **🖥️ Interactive TUI** — keyboard-driven dashboard (ratatui)
- **💾 SQLite cache** — cut redundant API calls and rate-limit pressure
- **🤖 AI analysis** — analyze, recommend, and rate difficulty via OpenAI or Anthropic
- **🔌 OpenAI tools / HTTP server** — expose capabilities to agents (bearer-authenticated)
- **🛡️ Security gate** — CVE, secret, license, and quality checks before you push
- **🪝 Git hooks** — install pre-push security as one command
- **📦 JSON everywhere** — scriptable, machine-readable output

---

## Install

Requires **Rust 1.88+**. Install from source:

```bash
git clone https://github.com/grarizki/foxes-howl.git
cd foxes-howl

# Install the binary on your PATH
cargo install --path .

# Or build a locally-optimized release binary
cargo build --release
```

### 1. Set a GitHub token

No token works, but rate limits are tight (60 req/h). A token gets you **5,000 req/h**:

```bash
export GITHUB_TOKEN="ghp_your_token_here"
```

### 2. Create the default config

```bash
gh-opportunities init
```

Writes `~/.config/gh-opportunities/config.toml` with sensible defaults for scoring weights, the AI provider, your profile, and security options. Tune it to your workflow.

### 3. Re-run the CI locally (optional but recommended)

```bash
cargo fmt --check   # formatting
cargo clippy        # lints
cargo test          # 109 unit tests
cargo audit         # dependency CVEs
cargo tarpaulin     # coverage report
```

> CI runs all of these on every pull request across Ubuntu, macOS, and Windows.

---

## Quick start

```bash
# Score good first issues in a repo
gh-opportunities scan serde-rs/serde

# Discover high-opportunity Rust repos
gh-opportunities discover --lang rust --min-stars 100

# Find issues nobody's touched in 30+ days
gh-opportunities stale tokio-rs/tokio

# Audit a repo's community health + code quality
gh-opportunities readme facebook/react
gh-opportunities quality rust-lang/rust

# Watch it all in the interactive TUI
gh-opportunities tui rust-lang/rust tokio-rs/tokio denoland/deno

# Repo health scores, ranked
gh-opportunities discover --lang python --json
```

Every scan supports `--json` for scripting.

---

## AI-Powered analysis

Set your provider key (OpenAI or Anthropic):

```bash
export OPENAI_API_KEY="sk-..."
# or
export ANTHROPIC_API_KEY="sk-ant-..."
```

```bash
# Summarize a repo's contribution landscape
gh-opportunities ai analyze rust-lang/rust

# Personalized recommendations from your skills
gh-opportunities ai recommend tokio-rs/tokio --skills "rust,async,cli" --hours 10

# Rate issue difficulty
gh-opportunities ai difficulty denoland/deno

# Skip the cost-confirmation prompt
gh-opportunities ai analyze rust-lang/rust --yes
```

A token-cost estimate is shown before each LLM call; confirm or pass `--yes`.

---

## Agent integration

Expose gh-opportunities to AI tools or a local service:

```bash
# Dump OpenAI function-calling tool definitions
gh-opportunities tools

# Execute a tool call directly
gh-opportunities call scan_issues --args '{"repo":"rust-lang/rust","limit":10}'

# Or run the HTTP server (binds 127.0.0.1, bearer-auth)
export GH_OPP_TOKEN="your-secret-token"
gh-opportunities serve --port 3737
```

HTTP endpoints: `GET /health`, `GET /tools`, `POST /call`, `POST /ai/analyze`, `POST /ai/recommend`, `POST /ai/difficulty`, `POST /security`, `GET /profile`.

---

## Security gate

```bash
# All checks
gh-opportunities security

# One check at a time
gh-opportunities security --check audit
gh-opportunities security --check secrets
gh-opportunities security --check quality
gh-opportunities security --check license

# JSON output, or auto-fix what's fixable (cargo fmt)
gh-opportunities security --json
gh-opportunities security --fix

# Install as a pre-push hook
gh-opportunities hooks install
```

The four checks: **audit** (cargo-audit CVEs), **secrets** (regex API/token/password scanner), **license** (cargo-deny / fallback metadata), **quality** (fmt + clippy + test). The pre-push hook **blocks the push if any check fails**.

---

## Scoring

**Issue score (0–1):** 0.5 label match (`good first issue` etc.) · 0.3 description > 50 chars · 0.2 unassigned.

**Repo composite (0–1):** 0.3 avg issue score · 0.2 stale ratio · 0.2 README/community gaps · 0.3 code-quality deficit. **A higher score means more high-upside openings.**

---

## Architecture

```
src/
├── main.rs            # CLI dispatch, table rendering
├── cli.rs             # clap derive definitions
├── config.rs          # XDG config, TOML parsing
├── db.rs              # SQLite cache (rusqlite)
├── github/            # octocrab client, issue fetch, discovery
├── analysis/          # stale / readme / code_quality / scoring
├── ai/                # provider trait, openai, anthropic, prompts, estimate, tools
├── security/          # audit, secrets, license, quality
├── serve/             # axum HTTP server + routes (bearer auth)
├── hooks/             # pre-push hook installer
└── tui/               # ratatui screens: dashboard, issues, repos, detail
```

**Modules are small, single-purpose, and testable.** The `ai/` and `security/` layers are clean seams — swap providers or add checks without touching callers.

---

## Contributing

We welcome contributors — this project is built specifically to help people land their first real open-source PR. Good starting points:

1. **Pick an issue** — look for `good first issue` / `help wanted` labels in this repo.
2. **Set up** — clone, `cargo install --path .`, run the smoke test above, `cargo test`.
3. **Keep it green** — commit through `prek` (fmt + clippy + test) and open a PR.
4. **Follow conventions** — each module owns its tests; add one alongside any non-trivial logic.

**Community conventions**
- Format: `cargo fmt` (pre-commit enforced)
- Lint: `cargo clippy` (pre-commit enforced)
- Tests: `cargo test`; run `cargo tarpaulin` for coverage
- Hooks: `prek` manages fmt/clippy/test; pre-push security gate optional

Ideas that would help most (see `cargo discover --language rust`):

- More scoring heuristics / prio-labels
- Webhook or scheduled-run support
- A CI-dashboard view
- Alternate LLM providers
- Parser for more license sources

Start small, ask questions, and open your first PR. The contribution loop here is the one this tool searches for.

---

## Tech stack

| Layer        | Crates                                       |
|--------------|----------------------------------------------|
| CLI          | clap 4 (derive)                             |
| TUI          | ratatui 0.30 + crossterm 0.28                  |
| Async        | tokio 1 (full)                              |
| HTTP client  | reqwest 0.12 + rustls                       |
| HTTP server  | axum 0.8                                    |
| DB / cache   | rusqlite 0.35 (bundled)                     |
| Analysis     | regex, chrono, serde / serde_json           |
| Config       | dirs (XDG), toml                           |
| Errors       | thiserror (lib) · anyhow (bin)              |
| Logging      | tracing / tracing-subscriber               |

**Toolchain:** Rust 2021 edition · MSRV 1.88 · LTO + strip in release · pre-commit hooks (`prek`) · GitHub Actions CI

---

## License

[MIT](LICENSE) © 2026. Use it, learn from it, extend it — and go make your first open-source contribution. 🚀