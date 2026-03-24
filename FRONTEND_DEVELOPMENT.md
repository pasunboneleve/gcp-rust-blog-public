Frontend Development
--------------------

Quickstart
==========

Preferred workflow
==================

Use the external `devloop` tool against this repo for the integrated
Rust + CSS + content + cloudflared loop. Repo-owned helper scripts:

- [`scripts/build-css.sh`](scripts/build-css.sh)
- [`scripts/current-post-slug.sh`](scripts/current-post-slug.sh)

Fallback direct repo workflow
=============================

In one terminal

```sh
direnv allow
```

then

```sh
bacon run
```

In another terminal

```
./scripts/build-css.sh
```

Dependencies
============

You'll need to install [bacon](https://dystroy.org/bacon/)

sh
```
cargo install bacon
```

and [tailwind](https://tailwindcss.com/).

sh
```
npm install -g @tailwindcss/cli
```

Although you really should use [bun](https://bun.com/) instead of [npm](https://docs.npmjs.com/cli/).

And finally, I like using [direnv](https://direnv.net/) to
automagically run Bash and add environment variables when we change
into a directory.

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

When `build-css.sh` recompiles `content/static/tailwind.css`, the Rust
file watcher detects the change and signals the browser to refresh via
WebSocket — no server restart needed. Bacon ignores changes to
`content/static/tailwind.css` for exactly this reason.
