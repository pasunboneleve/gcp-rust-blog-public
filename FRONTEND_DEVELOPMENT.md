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
- Markdown and HTML content reload without manual server restarts
- `cloudflared` is restarted when the workflow needs a fresh public URL
- the current browsed post URL is printed in a copy/paste-friendly form
  for LinkedIn, Slack, or X card validation

Repo-owned helper files:

- [`devloop.toml`](devloop.toml)
- [`scripts/build-css.sh`](scripts/build-css.sh)
- [`scripts/current-post-slug.sh`](scripts/current-post-slug.sh)

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

You'll need [`devloop`](https://github.com/pasunboneleve/devloop),
[Tailwind CLI](https://tailwindcss.com/), and optionally
[bacon](https://dystroy.org/bacon/) for the fallback path.

```sh
cargo install --git https://github.com/pasunboneleve/devloop.git
npm install -g @tailwindcss/cli
cargo install bacon
```

Or use [bun](https://bun.com/) instead of
[npm](https://docs.npmjs.com/cli/) if you prefer.

[`direnv`](https://direnv.net/) is also useful for automatically
loading environment variables when you change into the repo.

CSS Architecture
================

Rust generates HTML structure and emits **semantic class names only**.
All visual decisions — spacing, colour, size, weight — live in
`tailwind.css`. This means you can change how any element looks by
editing `tailwind.css` alone, with changes visible immediately in the
browser via hot reload (no server restart, no Rust recompile).

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

With `devloop`, CSS recompiles, content reloads, tunnel restarts, and
URL publication are orchestrated from one supervisor. Without it, you
would need to coordinate those loops yourself across separate terminal
sessions.
