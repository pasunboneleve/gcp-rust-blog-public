# AGENTS.md – Instructions for coding agents

## Purpose
This file defines implementation rules for AI agents working in this
repository. It exists to keep code reviews consistent and make changes
small, testable, and Rust-idiomatic.

## Scope and non-scope
- Scope: code changes for this repository, tests, and docs tied to
  implementation.
- Non-scope: product strategy changes and architecture rewrites that
  conflict with `README.md` and `PLAN.md` without explicit approval.

## Common Development Commands

Local development requires **two parallel terminals**:

```bash
# Terminal 1 — Rust server with restart-on-change and fresh cloudflared URL
bacon run

# Terminal 2 — Tailwind CSS watcher
./scripts/tailwatch.sh
```

`bacon run` executes `./scripts/cloudflared-bacon-run.sh`, so each code-triggered restart also replaces the tunnel and prints a fresh public `/posts/<slug>` URL.

Tailwind must be running alongside the server so that class changes in
`src/**/*.rs` and `content/**/*.html` are compiled into
`content/static/tailwind.css` automatically. Stopping `tailwatch.sh`
means CSS changes (including new Tailwind classes added to generated
HTML) will not appear in the browser.

## Success criteria
- New logic includes tests.
- Work is delivered as small, reviewable increments aligned with
  `PLAN.md` sequential steps.
- `cargo clippy` and `cargo test` pass before proposing changes.

## Project overview
Read `README.md` for product goals and `PLAN.md` for staged
execution.

### Content Management
- Blog posts are stored as Markdown files in `content/posts/`
- Create new posts by adding `<slug>.md` files in `content/posts/`
- Posts are accessible at `/posts/<slug>`
- Banner HTML is in `content/banner.html`

### GCP Deployment Commands
Set required environment variables first:
```bash
export PROJECT_ID={{GCP_PROJECT_ID}}
export GCP_REGION={{GCP_REGION}}
export SERVICE_NAME=blog
export REPO=blog
```

Build and deploy to Cloud Run:
```bash
# Build and push with Cloud Build
gcloud builds submit --project $PROJECT_ID \
  --tag $GCP_REGION-docker.pkg.dev/$PROJECT_ID/$REPO/$SERVICE_NAME:latest

# Deploy to Cloud Run
gcloud run deploy $SERVICE_NAME \
  --image $GCP_REGION-docker.pkg.dev/$PROJECT_ID/$REPO/$SERVICE_NAME:latest \
  --region $GCP_REGION --platform managed --allow-unauthenticated \
  --port 8080 --ingress all --project $PROJECT_ID
```

### Infrastructure Management
Bootstrap Terraform state (one-time):
```bash
PROJECT_ID={{GCP_PROJECT_ID}} GCS_BUCKET={{YOUR_TF_STATE_BUCKET}} ./scripts/bootstrap-tf-state.sh
```

Apply infrastructure:
```bash
cd infra
terraform init -backend-config="bucket={{YOUR_TF_STATE_BUCKET}}" -backend-config="prefix=gcp-rust-blog/infra"
terraform apply \
  -var="project_id={{GCP_PROJECT_ID}}" \
  -var="project_number={{GCP_PROJECT_NUMBER}}" \
  -var="pool_id={{GCP_WORKLOAD_IDENTITY_POOL}}" \
  -var="provider_id={{GCP_WORKLOAD_IDENTITY_PROVIDER}}" \
  -var="github_owner=<github_owner>" \
  -var="github_repo=gcp-rust-blog" \
  -var="cloud_run_url={{CLOUD_RUN_SERVICE_URL}}"
```

## Architecture Overview

### Application Structure
- **Single-file web server**: `src/main.rs` contains the entire Axum-based web application
- **Static content**: Uses Rust's axum framework to serve HTML and render Markdown posts
- **Content-driven**: Blog posts are Markdown files that get converted to HTML at request time
- **Minimal state**: Only loads banner HTML on startup, posts are read from filesystem per request

### Key Components
- **Axum router**: Handles HTTP routing with two main routes:
  - `/` - Homepage with welcome message and post links
  - `/posts/:slug` - Dynamic post rendering from Markdown files
- **Markdown processing**: Uses `pulldown-cmark` for Markdown to HTML conversion
- **Logging**: Configured with `tracing` and `tracing-subscriber` for structured logging

### Dependencies
- `axum 0.7` - Web framework
- `tokio` - Async runtime with full features
- `pulldown-cmark 0.10` - Markdown parser
- `tracing` ecosystem - Logging and observability

### Deployment Architecture
- **Cloud Run**: Containerized deployment on Google Cloud Platform
- **Load Balancer**: Global HTTP(S) load balancer for custom domain SSL support
- **GitHub Actions CI/CD**: Automated deployment via Workload Identity Federation
- **Artifact Registry**: Container image storage
- **Infrastructure as Code**: Terraform/OpenTofu for WIF setup and IAM roles

### Infrastructure Components
The `infra/` directory contains Terraform configuration for:
- Workload Identity Pool and Provider for GitHub OIDC authentication
- Service account IAM bindings for deployment permissions
- Required project-level roles: Cloud Run admin, Artifact Registry writer, Load Balancer admin
- Global HTTP(S) Load Balancer with SSL certificates for custom domain support
- DNS zone and records for domain management
- Network Endpoint Group (NEG) connecting load balancer to Cloud Run

### Content Structure
```
content/
├── banner.html          # Site header with navigation
└── posts/
    └── first-post.md    # Example blog post
```

## Environment Configuration
- `PORT` - Server port (default: 8080, required for Cloud Run)
- `RUST_LOG` - Log level (default: "info")

## Security Considerations
- Container runs as non-root user
- Uses minimal IAM permissions via dedicated service account
- Secrets managed via environment variables, not baked into images
- Public blog configured with `--allow-unauthenticated`

## Frontend CSS rule

Rust emits **semantic class names only** — no Tailwind utility classes
in format strings. All visual decisions live in `tailwind.css`.
See `FRONTEND_DEVELOPMENT.md` for the full pattern and the table of
defined component classes.

## Engineering standards
- No global state unless there is a clear documented need.
- No hardcoded configuration. Keep configuration in config files.
- NEVER vendor software.

## Aesthetic rules for adding content
- Read the recent posts before changing a new one so the new piece matches the established visual and editorial pattern.
- Track the work in `bd` as small tasks before editing: image selection/preparation first, then post updates, then any documentation follow-up.
- Prefer public-domain or equivalently reusable images from Wikimedia Commons when adding artwork to posts.
- Verify that the exact Wikimedia asset exists before committing to it. Open the Commons page, resolve the direct file URL, and confirm the image returns successfully instead of assuming the filename is correct.
- Choose images that are consonant with the argument of the post, look old or classical, and are recognisable as belonging to a distinct cultural tradition.
- If the original image is a poor fit for social cards, create a local derivative in `content/static/` and reference that derivative from the post front matter `image:` field.
- Social-card derivatives should be suitable for X and LinkedIn previews. Use a stable local asset, keep the aspect ratio close to `1200x630`, and crop rather than stretch.
- When an image has a predominantly white or washed-out background, it is acceptable to add a subtle warm yellow overlay so it reads as aged and remains legible in the site design.
- Reuse the same artwork family at the end of the post as a sourced figure with caption and attribution, even when the front matter points at a cropped social derivative.
- Convert structural diagrams in posts to Mermaid when they are meant to explain system shape or flow. Keep literal commands or code examples as fenced code.
- Write posts primarily as prose. Single-sentence paragraphs and stanza-like emphasis are acceptable, but they should be used deliberately to stress key points rather than dominate the structure of the piece.
- Preserve meaningful bullet lists from the draft when they clarify distinctions, examples, or recurring patterns. Do not flatten them into prose just for uniformity.

**MANDATORY WORKFLOW:**

Use 'bd' for task tracking.
Use $roborev:review after committing code changes and before rebasing.

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Commit work** - Always commit so roborev can check quality of
   work; commit --amend until the functionality is implemented
   correctly as verified by human and roborev.
3. **Update issue status** - Close finished work, update in-progress items
4. **Hand off** - Provide context for next session
