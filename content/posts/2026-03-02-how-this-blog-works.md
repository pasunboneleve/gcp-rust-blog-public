---
title: "How this blog works (and why it’s built this way)"
date: 2026-03-02
slug: 2026-03-02-how-this-blog-works
---

A blog is trivial.

Until you take the entire path seriously.

- local edit → browser refresh
- git push → build → deploy
- DNS → TLS → request routing
- content change → release artifact

This post describes <a href="https://github.com/pasunboneleve/gcp-rust-blog-public" target="_blank" rel="noopener noreferrer">the system as it exists <strong>today</strong></a>, and why each piece is there.

Not to run a blog.

To minimise the cost of change.

---

## Architecture at a Glance

```mermaid
flowchart TD

  USER["👤 Reader"]
  INTERNET["🌐 Internet + DNS + TLS"]
  CLOUDRUN["☁️ Cloud Run service<br/>(container running Axum app)"]
  APP["🦀 Rust application<br/>(Axum router + handlers)"]
  CONTENT["📝 content/*<br/>(Markdown + layout fragments)"]

  USER --> INTERNET --> CLOUDRUN --> APP --> CONTENT
```

**Users** here means actual readers — someone opening the site in a browser.

They do not interact with GitHub.\
They do not know about CI.

They send an HTTP request.\
The container responds with HTML.

Everything else exists to support that moment.

---

## Developer Loop (Local)

```mermaid
flowchart TD

  EDIT["✍️ Edit src/* or content/*"]
  RUN["🦀 cargo run<br/>(RUST_ENV=development)"]
  WATCH["👀 File watcher"]
  WS["🔌 Websocket reload signal"]
  BROWSER["💻 Local browser refresh"]

  EDIT --> RUN
  RUN --> WATCH
  WATCH --> WS
  WS --> BROWSER
```

In development mode:

- A file watcher monitors `content/`
- Changes trigger in-memory reload
- A WebSocket broadcasts `"reload"`
- The browser refreshes automatically

Edit. Save. See change.

No manual restart.
No rebuild for content tweaks.

Feedback loops stay short.

---

## Runtime Responsibilities

At startup, the application:

- Loads `content/layout.html`
- Loads `content/banner.html`
- Loads `content/home.md`
- Scans `content/posts/*.md`
- Parses YAML front matter
- Renders Markdown to HTML

There is no database.

Content lives in git.\
State lives in files.

A post is simply:

```markdown
---
title: "Post title"
date: 2026-03-02
slug: 2026-03-02-example
---

Markdown body here.
```

Rendering is deliberately simple:

- Markdown → HTML (tables + math enabled)
- Layout placeholders replaced (`{{ banner }}`, `{{ content }}`, `{{ posts }}`)
- Full HTML page returned

No ORM.\
No CMS.\
No runtime mutation.

Constraint keeps surface area small.


---

## Development Loop (Production)

```mermaid
flowchart TD

  PUSH["📤 git push to main"]
  GHA["⚙️ GitHub Actions"]
  AUTH["🔐 OIDC → Workload Identity Federation"]
  BUILD["🐳 Docker build"]
  AR["📦 Artifact Registry"]
  DEPLOY["🚀 Cloud Run deploy"]
  USERS["👥 Readers receive new version"]

  PUSH --> GHA
  GHA --> AUTH
  AUTH --> BUILD
  BUILD --> AR
  AR --> DEPLOY
  DEPLOY --> USERS
```


Push to `main`.

That is the release process.

GitHub Actions:

1. Authenticates to <a href="https://cloud.google.com/" target="_blank" rel="noopener noreferrer">Google Cloud (GCP)</a> using <a href="https://openid.net/connect/" target="_blank" rel="noopener noreferrer">OIDC</a>
(no stored service account keys)

2. Detects change type:
    - Full (code + content)
    - Content-only

3. Builds accordingly:

    **Full build**
    - Compile Rust
    - Build runtime image
    - Tag with commit SHA

    **Content-only build**
    - Overlay updated `content/`
    - Tag with commit SHA

4. Deploys to Cloud Run

Content changes ship without recompiling the binary.

That is deliberate.

Content is production change.\
It should not pay the full rebuild tax.

---

## Infrastructure

Infrastructure is declared in <a href="https://opentofu.org/" target="_blank" rel="noopener noreferrer">OpenTofu</a>.

It provisions:

- <a href="https://cloud.google.com/artifact-registry/docs" target="_blank" rel="noopener noreferrer">Artifact Registry</a>
- <a href="https://cloud.google.com/run/docs" target="_blank" rel="noopener noreferrer">Cloud Run service</a>
- <a href="https://cloud.google.com/iam/docs/workload-identity-federation" target="_blank" rel="noopener noreferrer">Workload Identity Federation</a>
- <a href="https://cloud.google.com/iam/docs/overview" target="_blank" rel="noopener noreferrer">IAM bindings</a> (least privilege)
- <a href="https://cloud.google.com/dns/docs" target="_blank" rel="noopener noreferrer">DNS records</a>

There are no manual console steps in steady state.

If something exists, it is declared.

If it is not declared, it does not exist.

---

## Security Posture

- No long-lived credentials in CI

- <a href="https://openid.net/connect/" target="_blank" rel="noopener noreferrer">OIDC</a> federation between GitHub and <a href="https://cloud.google.com/" target="_blank" rel="noopener noreferrer">GCP</a>

- Runtime container runs as non-root

- Multistage build to minimise attack surface

- Runtime does not require <a href="https://cloud.google.com/apis" target="_blank" rel="noopener noreferrer">GCP API</a> access

The container can serve traffic.

It cannot mutate infrastructure.

Boundaries matter.

---

## Why This Design?

Most blogs optimise for features.

This one optimises for:

- Reproducibility
- Short feedback loops
- Low deploy friction
- Visible state
- Minimal moving parts

The interesting problem is not publishing text.

It is reducing the cost of safe change.

This system demonstrates:

- Content treated as deployable artifact
- CI/CD without stored secrets
- Infra-as-code as baseline
- Developer ergonomics as design constraint

---

## What This Actually Signals

The blog itself is not the point.

The system is.

It shows how I think about:

- Platform as a product
- Flow as a measurable property
- Security as default posture
- Change as the primary unit of engineering work

When implementation becomes commoditised, leverage shifts to architecture.

This is a small system.

Small systems reveal principles clearly.

And principles scale.

<figure>
  <a href="https://commons.wikimedia.org/wiki/File:Li_Tang_-_Wind_in_Pines_Among_a_Myriad_Valleys.jpg" target="_blank" rel="noopener noreferrer">
    <img src="https://upload.wikimedia.org/wikipedia/commons/f/f3/Li_Tang_-_Wind_in_Pines_Among_a_Myriad_Valleys.jpg" alt="Whispering Pines in Myriad Valleys (宋李唐萬壑松風圖 軸), Li Tang" loading="lazy" style="display:block;margin:0.5rem auto;max-width:720px;width:100%;height:400px;object-fit:cover;object-position:50% 12%;outline:1px solid currentColor;outline-offset:2px;background:transparent;" />
  </a>
  <figcaption>
    <strong>Figure.</strong> Collection image from Li Tang's "Whispering Pines in Myriad Valleys" (宋李唐萬壑松風圖　軸).
    Source: <a href="https://commons.wikimedia.org/wiki/File:Li_Tang_-_Wind_in_Pines_Among_a_Myriad_Valleys.jpg" target="_blank" rel="noopener noreferrer">Wikimedia Commons</a>.
  </figcaption>
</figure>
