[![GCP Rust Blog CI/CD](https://github.com/pasunboneleve/gcp-rust-blog-public/actions/workflows/deploy.yml/badge.svg)](https://github.com/pasunboneleve/gcp-rust-blog-public/actions/workflows/deploy.yml)

# From Commit to Production, Automatically

This repository is a small Rust web application designed so that every change flows from commit to production without manual intervention.

Infrastructure, CI/CD, and security are defined alongside the code and evolve with it as a single system. There are no hidden steps and no out-of-band processes.

The focus is not the technology itself, but the shape of the system: making correct change the path of least resistance.

## Quick Start

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) for building and
  running the app locally. Install with `rustup`, which provides
  `cargo` and `rustc`.
- [devloop](https://github.com/pasunboneleve/devloop) for the primary
  local development workflow. Install with
  `cargo install --git https://github.com/pasunboneleve/devloop.git`.
- [cloudflared](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/)
  for the shareable public tunnel used to validate social cards during
  local development. Install it from Cloudflare's package or binary
  distribution for your platform.
- [Node.js](https://nodejs.org/) or [Bun](https://bun.com/) to install
  the Tailwind CLI.
- [Tailwind CLI](https://tailwindcss.com/docs/installation/tailwind-cli)
  for CSS compilation during local development. Install with
  `npm install -g @tailwindcss/cli` or the equivalent Bun workflow.
- [Chrome DevTools MCP](https://www.npmjs.com/package/chrome-devtools-mcp)
  if you want an MCP client to inspect the devloop-managed Chromium
  instance. It runs through `npx` and requires Node.js 22.12 or newer.
- [direnv](https://direnv.net/) to auto-load repo environment variables.
  Install it from your system package manager and run `direnv allow` in
  the repo.
- [bacon](https://dystroy.org/bacon/) only if you want the fallback
  direct-repo workflow instead of `devloop`. Install with
  `cargo install bacon`.
- Docker for containerization
- `gcloud` CLI configured with your GCP project
- OpenTofu/Terraform for infrastructure management
- [`dress-rehearsal`](https://github.com/pasunboneleve/dress-rehearsal)
  if you want to run isolated infrastructure create/destroy rehearsals

### Configuration Setup

1. **Prepare deployment variables if you need them locally**:
   ```bash
   cp .env.template .env
   direnv allow
   ```
   Edit `.env` with your actual GCP project values. The checked-in
   `.envrc` loads `.env`, exports Terraform inputs as `TF_VAR_*`, and
   renders backend config for `infra/immutable` and `infra/testable`.

2. **Create GitHub Personal Access Token**:
   - Go to [GitHub Settings > Developer settings > Personal access tokens > Tokens (classic)](https://github.com/settings/tokens)
   - Click **"Generate new token (classic)"**
   - Set **Expiration** as needed (e.g., 90 days)
   - Select **repo** scope (full control of private repositories)
   - Click **"Generate token"** and copy the token
   - Add the token to your `.env` file as `GITHUB_TOKEN`, or authenticate
     with `gh auth login` and let `.envrc` refresh it

   ⚠️ **Important**: Store this token securely - GitHub won't show it again!

### Local Development

Recommended:

- use the external `devloop` supervisor for the full Rust + CSS +
  content + cloudflared workflow
- use the repo-local [`devloop.toml`](devloop.toml) as the working client config
- keep repo-specific helper scripts in [`scripts/build-css.sh`](scripts/build-css.sh)
  and [`scripts/current-browser-path.sh`](scripts/current-browser-path.sh)

Primary local workflow:

```bash
direnv allow
devloop run
```

That gives you one supervised loop for Rust rebuilds, content-triggered
server restarts, CSS recompilation, browser refresh notifications, and
copy/paste-ready public post URLs for card validation.

Devloop also starts a visible Chromium instance with Chrome DevTools
remote debugging on `http://127.0.0.1:9222`. To let an MCP client use
that browser, add the Chrome DevTools MCP server to the client and point
it at the running Chromium instance:

```bash
codex mcp add chrome-devtools -- npx chrome-devtools-mcp@latest --browser-url=http://127.0.0.1:9222
```

For clients configured with JSON, use the same command and arguments:

```json
{
  "mcpServers": {
    "chrome-devtools": {
      "command": "npx",
      "args": [
        "chrome-devtools-mcp@latest",
        "--browser-url=http://127.0.0.1:9222"
      ]
    }
  }
}
```

Run `devloop run` before asking the MCP client to inspect the page. The
debugging port is local but powerful; do not browse sensitive sites in
that Chromium window while MCP access is enabled.

Fallback direct-repo workflow:

```bash
# Terminal 1: Rust server with restart-on-change
bacon run

# Terminal 2: one-shot CSS build when needed
./scripts/build-css.sh

# Check and format code
cargo check
cargo fmt
cargo clippy
```

### Content Management
- **Blog posts**: Add Markdown files in `content/posts/` as `<slug>.md`
- **Access posts**: Visit `/posts/<slug>` in your browser
- **Site header**: Customize `content/banner.html` for navigation and branding

## Project Structure
```
├── src/                     # Axum app, content loading, markdown, metadata
├── content/
│   ├── banner.html          # Site header with navigation
│   ├── layout.html          # Shared page shell
│   ├── home.md              # Home page content
│   └── posts/
│       └── <slug>.md        # Blog post content
├── docs/                    # Architecture, security, and infrastructure docs
├── infra/
│   ├── immutable/           # State for resources unsafe to recreate
│   ├── testable/            # State for rehearseable infrastructure
│   └── README.md            # Infrastructure workflow notes
├── scripts/
│   ├── dress-testable.sh    # Isolated dress rehearsal wrapper
│   └── render-backend-config.sh
├── .github/workflows/       # CI/CD automation
└── Dockerfile               # Multi-stage container build
```

## Architecture

This project implements a **cloud-native, security-first architecture**:

- **Application**: Modular Rust web server using Axum
- **Content**: File-based blog posts in Markdown format
- **Rendering**: Markdown, KaTeX math, Mermaid diagrams, and social metadata
- **Infrastructure**: Managed with two OpenTofu/Terraform roots:
  `infra/immutable` for resources that must not be destroyed and recreated,
  and `infra/testable` for resources that can be rehearsed with alternate
  names
- **Deployment**: Automated CI/CD with GitHub Actions and Workload Identity
  Federation
- **Security**: Least-privilege service accounts and non-root containers
- **DNS**: Managed through Google Cloud DNS with OpenTofu

At startup, the app loads site configuration, HTML templates, the home
page, the 404 page, and post Markdown from `content/`. Requests are
served from that loaded content plus static assets under
`content/static/`.

Infrastructure rehearsals use
[`dress-rehearsal`](https://github.com/pasunboneleve/dress-rehearsal), a tool
that applies and destroys a deployment root in isolated local state. In this
repo, run it through `scripts/dress-testable.sh`; the wrapper points dress at
`infra/testable` and replaces production resource names with run-scoped names.
Do not run dress against `infra/immutable`.

## Getting Started

To deploy your own instance:

1. **Fork this repository**
2. **Configure infrastructure**:
   ```bash
   cp .env.template .env
   direnv allow
   ```
3. **Deploy infrastructure**:
   ```bash
   tofu -chdir=infra/testable init -backend-config=backend.auto.hcl
   tofu -chdir=infra/immutable init -backend-config=backend.auto.hcl
   tofu -chdir=infra/testable plan
   tofu -chdir=infra/immutable plan
   ```
   For this production project, import existing resources into the matching
   root before applying. `infra/immutable` owns service accounts, WIF, DNS,
   GitHub secrets, IAM grants, and other resources that should not be
   destroyed and recreated. `infra/testable` owns Artifact Registry and load
   balancer resources that can be rehearsed with alternate names through
   `scripts/dress-testable.sh`.
4. **Deploy**: Push to main branch triggers automatic deployment

### CI/CD Build Modes

The deploy workflow uses two build modes:

- **Full build**: Runs when application/infrastructure code changes. Builds and pushes:
  - `:${GITHUB_SHA}` (deploy image)
  - `:app-base` (binary/runtime base image)
- **Content-only build**: Runs when all changed files are under `content/`.
  - Reuses `:app-base`
  - Builds a lightweight overlay image that only updates `/app/content`
  - Pushes `:${GITHUB_SHA}` and deploys it

If `:app-base` does not exist yet, the workflow automatically falls back to a full build and publishes it for future content-only deployments.

## Documentation

- **[Security Architecture](docs/SECURITY.md)** - Service accounts, IAM, and
  security best practices
- **[Infrastructure Guide](docs/INFRASTRUCTURE.md)** - Deployment, DNS, and
  infrastructure management

## Key Features

✅ **Infrastructure as Code** - Everything managed with OpenTofu\
✅ **Secure CI/CD** - GitHub Actions with Workload Identity Federation\
✅ **Least Privilege** - Dedicated service accounts with minimal permissions\
✅ **Automated DNS** - Domain management through code\
✅ **Container Security** - Multi-stage builds, non-root containers\
✅ **Observability** - Structured logging with `tracing`\
✅ **Faster content deploys** - Content-only changes reuse a prebuilt app base image

## Environment Configuration
- `PORT` - Server port (default: 8080, required for Cloud Run)
- `RUST_LOG` - Log level (default: "info")

## License
This project is open source and available under the MIT License.
