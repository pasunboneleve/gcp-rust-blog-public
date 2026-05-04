Frontend Development
--------------------

Quickstart
==========

Preferred workflow
==================

Use [`devloop`](https://github.com/pasunboneleve/devloop) against this
repo for the integrated Rust + CSS + content + cloudflared loop. This
is the primary development workflow for
[`gcp-rust-blog-public`](https://github.com/pasunboneleve/gcp-rust-blog-public).

From the repo root:

```sh
direnv allow
devloop run
```

What you get:

- Rust rebuilds and restarts in order when `src/**/*.rs` changes
- Tailwind recompiles when CSS-relevant files change
- Markdown and HTML content changes restart the server through `devloop`,
  so the browser sees the same state as a fresh startup
- the current browsed post URL is printed in a copy/paste-friendly form
  for LinkedIn, Slack, or X card validation
- a visible Chromium instance is started with Chrome DevTools remote
  debugging on `http://127.0.0.1:9222` for MCP-assisted inspection

Repo-owned helper files:

- [`devloop.toml`](devloop.toml)
- [`scripts/build-css.sh`](scripts/build-css.sh)
- [`scripts/current-browser-path.sh`](scripts/current-browser-path.sh)

Chrome DevTools MCP
===================

The primary workflow includes a devloop-managed Chromium process named
`chromium-mcp`. It opens the local blog page after the Rust server is
ready and exposes Chrome DevTools on `http://127.0.0.1:9222`.

Configure your MCP client once so it connects to that running browser:

```bash
codex mcp add chrome-devtools -- npx chrome-devtools-mcp@latest --browser-url=http://127.0.0.1:9222
```

Equivalent JSON configuration:

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

Start `devloop run` before using MCP tools. The remote debugging port
lets local tools inspect and control the Chromium window, so keep that
window for development pages and avoid sensitive browsing in it.

Fallback direct repo workflow
=============================

Without `devloop`, the same loop is much clumsier. You would need three
or four terminals for the server, CSS watcher, and tunnel management,
and every time the tunnel URL changed you would also have to rebuild the
full cloudflared URL plus post path manually before validating cards in
LinkedIn, Slack, or X. `devloop` turns that into a simple copy/paste.

```sh
direnv allow
bacon run
./scripts/build-css.sh
```

Dependencies
============

See the local development prerequisites in
[`README.md`](README.md). The important distinction is:

- `devloop`, `cloudflared`, Node or Bun, Tailwind CLI, and `direnv` are
  part of the primary workflow
- `chrome-devtools-mcp` is part of the MCP-assisted primary workflow and
  runs through `npx` from the MCP client
- `bacon` is fallback-only

CSS Architecture
================

Rust generates HTML structure and emits **semantic class names only**.
All visual decisions — spacing, colour, size, weight — live in
`tailwind.css`. This means you can change how any element looks by
editing `tailwind.css` alone. `devloop` recompiles the stylesheet and
then tells the browser to refresh, without a Rust recompile.

### The rule

**Never** place Tailwind utility classes directly in Rust format strings.
Instead:

1. Add a named class to `tailwind.css` using `@layer components` +
   `@apply`, or plain CSS for anything needing `color-mix()` or other
   non-`@apply`-able features.
2. Emit only that class name from Rust.

```rust
// ✗ Wrong — visual decisions in Rust
format!("<span class=\"text-xs text-base01 opacity-60 mt-0.5\">{s}</span>")

// ✓ Right — Rust assigns identity, CSS assigns appearance
format!("<span class=\"sidebar-post-subtitle\">{s}</span>")
```

### Defined component classes

All component classes for dynamically generated HTML are in `tailwind.css`,
grouped and labelled by section:

| Class | Element | Section |
|---|---|---|
| `.sidebar-post-link` | `<a>` wrapping each sidebar entry | Sidebar |
| `.sidebar-post-title` | Post title span in sidebar | Sidebar |
| `.sidebar-post-subtitle` | Post subtitle span in sidebar | Sidebar |
| `.post-header` | `<header>` wrapping post title area | Post page |
| `.post-eyebrow` | Flex row holding role badge + subtitle | Post page |
| `.post-role` | Role badge (e.g. "mechanism") | Post page |
| `.post-eyebrow-subtitle` | Subtitle span next to role badge | Post page |
| `.post-date` | Publication date line | Post page |

### When to use @apply vs plain CSS

- Use `@layer components { @apply ... }` for classes that compose
  standard Tailwind utilities (colours, spacing, typography).
- Use plain CSS for anything requiring `color-mix()`, `calc()`, or
  other CSS features that `@apply` cannot express.

### How hot reload works

With `devloop`, CSS recompiles, content-triggered server restarts,
browser refresh notifications, and URL publication are orchestrated from
one supervisor. The browser listens to `devloop`'s SSE reload stream;
the app no longer runs its own content watcher or reload websocket.
Without `devloop`, you would need to coordinate those loops yourself
across separate terminal sessions.
