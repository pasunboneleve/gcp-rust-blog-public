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
- [direnv](https://direnv.net/) to auto-load repo environment variables.
  Install it from your system package manager and run `direnv allow` in
  the repo.
- [bacon](https://dystroy.org/bacon/) only if you want the fallback
  direct-repo workflow instead of `devloop`. Install with
  `cargo install bacon`.
- Docker for containerization
- `gcloud` CLI configured with your GCP project
- OpenTofu/Terraform for infrastructure management

### Configuration Setup

1. **Set up environment variables**:
   ```bash
   cp .env.template .env
   # Edit .env with your actual GCP project values
   ```
   📝 **Template file**: [`.env.template`](.env.template)

2. **Configure infrastructure variables**:
   ```bash
   cp infra/prod.tfvars.template infra/prod.tfvars
   # Edit infra/prod.tfvars with your GCP project details
   ```
   📝 **Template file**: [`infra/prod.tfvars.template`](infra/prod.tfvars.template)

3. **Create GitHub Personal Access Token**:
   - Go to [GitHub Settings > Developer settings > Personal access tokens > Tokens (classic)](https://github.com/settings/tokens)
   - Click **"Generate new token (classic)"**
   - Set **Expiration** as needed (e.g., 90 days)
   - Select **repo** scope (full control of private repositories)
   - Click **"Generate token"** and copy the token
   - Add the token to your `infra/prod.tfvars` file as `github_token`

   ⚠️ **Important**: Store this token securely - GitHub won't show it again!

### Local Development

Recommended:

- use the external `devloop` supervisor for the full Rust + CSS +
  content + cloudflared workflow
- use the repo-local [`devloop.toml`](devloop.toml) as the working client config
- keep repo-specific helper scripts in [`scripts/build-css.sh`](scripts/build-css.sh)
  and [`scripts/current-post-slug.sh`](scripts/current-post-slug.sh)

Primary local workflow:

```bash
direnv allow
devloop run
```

That gives you one supervised loop for Rust rebuilds, CSS recompilation,
content reloads, cloudflared restarts, and copy/paste-ready public post
URLs for card validation.

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
├── src/main.rs              # Single-file Axum web server
├── content/
│   ├── banner.html          # Site header with navigation
│   └── posts/
│       └── first-post.md    # Example blog post
├── infra/                   # OpenTofu/Terraform infrastructure
├── .github/workflows/       # CI/CD automation
└── Dockerfile              # Multi-stage container build
```

## Architecture

This project implements a **cloud-native, security-first architecture**:

- **Application**: Single-file Rust web server using Axum framework
- **Content**: File-based blog posts in Markdown format
- **Infrastructure**: Fully managed with OpenTofu/Terraform
- **Deployment**: Automated CI/CD with GitHub Actions and Workload Identity
  Federation
- **Security**: Least-privilege service accounts and organization policies
- **DNS**: Managed through Google Cloud DNS with OpenTofu

## Getting Started

To deploy your own instance:

1. **Fork this repository**
2. **Configure infrastructure**:
   ```bash
   cp infra/prod.tfvars.template infra/prod.tfvars
   # Edit infra/prod.tfvars with your GCP project details and GitHub token
   ```
3. **Deploy infrastructure**:
   ```bash
   cd infra
   tofu init
   tofu apply -var-file="prod.tfvars"
   ```
   This automatically:
   - Sets up GCP Workload Identity Federation
   - Configures all required GitHub repository secrets
   - Provisions infrastructure components
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
