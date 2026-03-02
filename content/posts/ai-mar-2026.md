---
title: "Dr Strangelove or: How I learned to stop worrying and love AI"
date: 2026-03-02
slug: ai-mar-2026
---

<figure>
  <img src="https://upload.wikimedia.org/wikipedia/commons/8/85/Dr._Strangelove_-_The_War_Room.png" alt="The War Room scene from Dr. Strangelove" loading="lazy" />
  <figcaption>
    <strong>Figure 1.</strong> The War Room in <em>Dr. Strangelove</em>, a visual metaphor for systemic risk and escalation.
    Source: <a href="https://commons.wikimedia.org/wiki/File:Dr._Strangelove_-_The_War_Room.png" target="_blank" rel="noopener noreferrer">Wikimedia Commons</a> (public domain).
  </figcaption>
</figure>

When I joined an AI startup in 2021, I expected to find lots of
machine learning. It was the flavour of the month. Every company was
“AI-powered.” I assumed I would walk into a codebase dense with
models, training pipelines, feature engineering, and statistical
nuance.

Instead, I found something much simpler. There were small learned
models in specific places. But most of the system was composed of
rules, transformations, heuristics, and well-structured pipelines. It
wasn’t artificial general intelligence hiding in production.

It was mostly engineering. That surprised me.

Not because it was underwhelming — but because it was clarifying. The
real leverage wasn’t in exotic models. It was in how components were
composed, how data flowed, and how clearly responsibilities were
defined.

That was my first shift in perspective.

---

## The First Stage: Acceleration

A few years later, the next wave arrived: AI-assisted coding. Tools
like <a href="https://github.com/features/copilot" target="_blank" rel="noopener noreferrer">Copilot</a> began generating entire functions, tests, even small
modules. The effect was immediate. This is the first stage of AI
adoption:

**Use it to code faster.**

And it works. Boilerplate disappears. Endpoints materialise
quickly. Tickets close sooner. It feels like velocity has doubled.

But acceleration has a shape. When AI is used primarily as an
autocomplete engine, it optimises for local correctness. It solves the
immediate prompt. It does not optimise for global coherence across a
system. Files grow. Patterns repeat. Similar abstractions diverge
slightly. The system functions — but its internal logic becomes harder
to see.

The cost isn’t writing code. The cost is evolving it.

---

## The Second Stage: Friction

But after a few cycles, a different pattern emerges. Time to *write*
a feature decreases. Time to *integrate* a feature increases. Builds
grow slower. More files are touched per change. Code reviews take
longer because reviewers must reconstruct intent from generated
structure. Seemingly local changes trigger failures in distant parts
of the system. The surface area expands faster than understanding.

This is where the real cost appears.

AI-generated code often optimises for solving the immediate prompt. It
does not account for long-term architectural coherence. It does not
consolidate abstractions unless asked. It does not remove duplication
unless instructed. It does not feel the cumulative weight of a growing
system. So complexity accumulates. Not in dramatic failures — but in
friction:

- Longer CI times.
- Broader dependency graphs.
- More brittle tests.
- Bugs that are not confined to the feature location, but systemic.
- Reviews that focus on deciphering structure rather than evaluating
  behaviour.

---

## The Third Stage: Amplification

The real shift happens when AI stops being a typing assistant and
becomes a thinking tool. Instead of asking:

> “Write a controller that does X.”

You start asking:

> “What architectural options isolate this responsibility?”
>
> “Where might coupling emerge in this design?”
>
> “If this grows 10x, what breaks first?”

This is the second stage of adoption — not acceleration, but
amplification. AI becomes a way to explore design space. To simulate
trade-offs. To pressure-test structure before committing to it. It
does not replace engineering judgment. It sharpens it.

---

## What Changed for Me

I stopped worrying about AI replacing engineers. Because typing speed
was never the constraint. The constraint was always:

- clarity
- structure
- evolvability
- cost of change

AI makes code cheaper to produce. That makes architecture more
valuable, not less. When generation becomes abundant, coherence
becomes scarce. And coherence is a human responsibility.

---

## The Strangelove Moment

In <a href="https://www.imdb.com/title/tt0057012/" target="_blank" rel="noopener noreferrer">Dr Strangelove</a>, the danger wasn’t the machine itself. It was the
system built around it.

AI is similar. Used naively, it increases
entropy faster than we can contain it. Used deliberately, it
compresses feedback loops and expands the space of ideas we can
explore. I no longer worry about AI.

<figure>
  <img src="https://upload.wikimedia.org/wikipedia/commons/b/bb/Dr._Strangelove.png" alt="Dr. Strangelove close-up (Peter Sellers)" loading="lazy" style="display:block;margin:0 auto;" />
  <figcaption>
    <strong>Figure 2.</strong> Dr. Strangelove (Peter Sellers): “It’s simple to understand. And completely credible and convincing.”
    Source: <a href="https://commons.wikimedia.org/wiki/File:Dr._Strangelove.png" target="_blank" rel="noopener noreferrer">Wikimedia Commons</a> (public domain). Quote reference: <a href="https://en.wikiquote.org/wiki/Dr_Strangelove" target="_blank" rel="noopener noreferrer">Wikiquote</a>.
  </figcaption>
</figure>

I pay attention to how we
choose to use it. Because the future of engineering will not be
determined by how fast we can generate code. It will be determined by
how intentionally we shape systems.
