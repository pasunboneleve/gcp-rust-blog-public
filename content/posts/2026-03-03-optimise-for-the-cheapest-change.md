---
title: "Optimise for the Cheapest Change"
date: 2026-03-03
slug: 2026-03-03-optimise-for-the-cheapest-change
---

Most engineers already agree with Dan North’s idea of building the <a href="https://dannorth.net/blog/best-simple-system-for-now/" target="_blank" rel="noopener noreferrer">best simple system for now</a>. It is solid advice: avoid speculative architecture, ship the thing you need, and keep today’s constraints visible.

Where teams still get stuck is what happens next. The first system works, but replacing or reshaping it later is slow and risky.

## The hidden failure mode

<div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1.25rem; align-items: start;">
  <div style="min-width: 0; text-align: justify; text-justify: inter-word;">
    <p>A common sequence is straightforward:</p>
    <ol>
      <li>Avoid premature abstraction.</li>
      <li>Build a simple system.</li>
      <li>Ship it.</li>
      <li>Discover that further change is expensive.</li>
    </ol>
    <p>That does not always happen because the original design was wrong. It often happens because the <em>process</em> of change is costly: feedback loops are long, deployment is fragile, boundaries are fuzzy, and local edits trigger non-local consequences.</p>
    <p>The practical failure is not “we built something simple.” It is “we built something expensive to modify.”</p>
  </div>
  <figure style="margin: 0;">
    <img src="/static/changing-a-simple-system.png" alt="Changing a simple system can still be costly when process and dependencies are tangled" loading="lazy" style="display: block; width: 80%; height: auto; margin: 0 auto;" />
    <figcaption style="width: 80%; margin: 0.5rem auto 0; text-align: center;">
      <strong>Figure 1.</strong> Changing a simple system can still be costly.
    </figcaption>
  </figure>
</div>

## Shift the optimisation target

Instead of optimising for the best system, optimise for the cheapest change.

This is not a call for more abstraction or hypothetical extension points. It is a call for operationally cheap change:

- short feedback loops
- localised change surfaces
- explicit boundaries with minimal hidden state
- minimal manual release steps
- deployment that feels routine

That is the platform concern in plain terms: making safe change normal.

## What this looks like in practice

This blog is intentionally shaped for local, low-risk edits.

- Posts are plain Markdown files.
- Most edits touch one file.
- Hot reload makes local feedback immediate.
- Push to `master` deploys automatically, typically in about 90 seconds.

<figure>
  <img src="/static/cheapest-change-feedback-loop.svg" alt="Diagram of a cheap change loop from edit to preview to commit to deploy and back" loading="lazy" />
  <figcaption>
    <strong>Figure 2.</strong> Cheap change is mostly loop design: small edits, fast feedback, routine deploys.
  </figcaption>
</figure>

The product of this system is published writing, and AI helpers are part of that workflow. The point is not that AI exists; the point is that the system gives AI a safe operating surface. When artifacts are local and readable, changes are easy to verify and easy to review.

AI tends to amplify systems that are already coherent. In tangled systems, it tends to amplify the tangle.

## What this means for platform work

The same pattern holds in larger systems.

Change gets expensive when adding one feature touches five subsystems, when builds and tests are slow, when reviews are noisy, or when deployment requires coordinated rituals.

If you care about throughput over time, high-leverage platform work is reducing the cost of safe modification. That usually pays off faster than adding another abstraction layer.

## Not the same as premature abstraction

Premature abstraction asks, “What flexibility might we need?”

Cheapest-change thinking asks, “What change will definitely come, and how do we make that change safe and fast?”

The second question is easier to reason about because it is tied to observable flow, not speculation.

## Closing

Systems evolve, requirements move, and tools get replaced. None of that is surprising.

The useful design question is how painful those transitions are.

Build the best simple system for now, then shape it so replacing it later is cheap.
