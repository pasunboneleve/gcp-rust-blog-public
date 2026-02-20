---
title: "Dr Strangelove or: How I learned to stop worrying and love AI"
date: 2026-02-19
slug: ai-feb-2026
---

# The Stone Age

## An AI in the Sky

This story starts with me being hired as yet another PhD to work at an
AI startup. For real! I do not have a background in neural networks,
and my PhD, while in Neuroscience research, was not in computational
neural models. I was one of those researchers who try to reverse
engineer the brain by damaging it and checking what stopped
working. Kind of like how software engineers reverse engineers
systems, really.

But no surprise. While the startup had impressive code, it was not
strictly using neural networks. It was a very smart and pragmatic use
of business rules and pattern matching, with some machine learning
layered over it. Nice UI, too. I mean, it was an impressive
technology. But AI it was not. So I developed skepticism towards all
overblown claims about AI changing the world. I mean, I saw inside it,
and... it was lots of `if then else` and `loops`.

# The Bronze Age

## AI Smell

Over 2024 my teammates started using **GitHub Copilot** as a code
completion aid. From the start it displeased me. It gave them an
undeniable edge on speed. They could churn out features while I was
still reading code. However, the avalanche of code they created was
hardly clean. They themselves could not explain their choices. The
files were enormous. And nothing was abstracted. Piles and piles of
boilerplate repeated everywhere, but not exactly, which made it very
difficult to decide after the fact what the business logic was, or how
to abstract sensibly.

Guess who refactored the mess and added the abstractions? Yes, yours
truly. Which, do not misunderstand me, I was happy to do. As long as
they could be the domain experts, I could be the code janitor. It was
a good fit, and kept me safe from the pressure to deliver urgent
features.

Actually, it also kept me safe from redundancy. The coworkers who used
**Copilot** were the first to be made redundant. The company understood
that their features had to be implemented twice: once as a demo, and
once as production-grade code, and that was unacceptable. I felt sorry
for them, but again, that made me wary of using AI.

# The Iron Age

## AI Kills Stack Overflow

In 2025 **Google** started giving AI overviews instead of having
**Stack Overflow** links as the first search result. Including
code, which unsurprisingly was harvested from **Stack Overflow**. I
can't argue with **Google**, I had work to do, so I would use it as a
source of ideas and keep working.

Later this year a new colleague joined the team. As far as I know, he
did not use **Copilot** autocomplete. But despite seeming younger and
less experienced,

1. He made sound, pragmatic, simple, clean **programming pattern** and
   **library** choices;
2. He also churned through issues, but not to create cruft. He
   actively **removed** cruft. He seemed to see things from a much
   higher level than I ever experienced. It was breathtaking.
3. He started and finished working websites over a weekend and a
   working day. For fun. I mean, I could not compete with this guy in my
   wildest dreams.

At this point I was ready to concede defeat. I did not know what this
guy was smoking, but I wanted some. And then I was made redundant,
unsurprisingly.


# The Atomic Age

## Make Straight the Way of AI

I started applying. Frequently. I basically spent my waking hours
applying for jobs. I didn't waste time. I had years of resumes on
file, so I just updated them and asked AI to write draft cover letters
to address job advertisements (*copy/paste* 😹). I would then clean
up hallucinated experience and send it through. No time to wait.

I did land some good interviews. And I failed pretty quick at
in-person coding interviews. Not because I was ignorant. Not because I
was slow. Because looking up **Stack Overflow** was part of my
workflow, and using **AI** was not.

So I got the memo, compared different **AI** provider price/value
tradeoffs, and decided to use [Warp](https://www.warp.dev/) over a
week for free, to gain experience. This website, all of it, was
written in that week. I spent two days writing the **CI/CD** logic,
the [Terraform](https://developer.hashicorp.com/terraform) code and
the [Rust](https://rust-lang.org/) web server. And then another two
days polishing it.

That was impressive. There were hiccups: the **Rust** logic relied on
latest versions of libraries, but the **AI** guessed old syntax, and I
had to fix that by hand. Still, I had never implemented an end-to-end
website hosted in the cloud that fast! And mostly mindlessly! The hype
was real.

## From Here to Eternity

This is where my **AI** knowledge goes hockey-stick-shaped. I experimented
many different packages for using **AI** in my IDE. I experimented
with [BMAD method](https://github.com/bmad-code-org/BMAD-METHOD). I
tested **Gemini**, **Grok**, **Claude**, **DeepSeek**, **ChatGPT**. I
used **n8n** briefly. I tested different combinations of all of those
for different coding and documentation tasks, and different priced and
powered models under each.

From that I learned that **models** and **programming languages**
interact to waste time or make a developer productive.

### Why Programming Language Matters

The AI model relies on feedback to do its work. We developers do
too. Currently developers rely on information coming from our IDEs
(i.e., LSP) to identify syntax errors. The AI does not have that. Yes,
you could set up linting. But the AI normally gets confused between
errors of syntax and errors because the feature it wants to use is not
available in the library version you are using. And it things
misleading errors are pointing the right way. Which makes confusion
compound.

This is true of weakly typed languages such as
[Python](https://www.python.org) and
[TypeScript](https:://www.typescriptlang.org). Not so with
[Rust](https://rust-lang.org)! **Rust** is famous for giving helpful
error messages. Syntax errors lead to compilation failure, so if we
use a CLI tool that both writes to files and attempts to compile, the
AI will sort syntax errors without help. Finally, mismatched types
also lead to compilation failures. That's even before it runs tests!

**Rust** also has a famously extensive and difficult programming
API. What do I care, I ask? I'm not writing the code, the AI is! The
complexity ceased to be a barrier, how fast the AI can compile and get
feedback is the real barrier. And **Rust** shines here.

### Why Models Matter

This is an easy one. Let's pitch GPT-4o against Gemini 2.5 Pro. GPT
achieved 72.7% success on standard programming tasks, against 69.1% by
Gemini. Gemini is generally 50% cheaper than GPT. So at face value we
would choose Gemini and save a lot of money, right? Not so fast!

Assuming a typical coding project with **100 tasks** and a blended
token cost (roughly 3:1 input-to-output ratio), **GPT-4o** (via
ChatGPT) costs about **$4.38 per million tokens** ($2.50 input /
$10.00 output), while **Gemini 2.5 Pro** is cheaper at around **$3.44
per million tokens** ($1.25 input / $10.00 output for prompts ≤200k
tokens). This gives Gemini an upfront ~20–25% savings per
token. However, when accounting for iteration-driven errors—where
Gemini's slightly lower first-try success rate requires ~5 iterations
on failed tasks versus GPT-4o's ~1—the total effort balloons.

For simplicity, suppose a baseline successful task uses **T** tokens,
and each failed iteration adds another **T** (due to growing context
from error logs). With GPT-4o succeeding first-try on ~73% of tasks
(per earlier SWE-bench context), the expected tokens per task is:

\[ E_{\text{GPT}} = 0.73 \times T + 0.27 \times 5T = 2.08T \]

For Gemini at ~69% success:

\[ E_{\text{Gemini}} = 0.69 \times T + 0.31 \times 5T = 2.24T \]

This yields a ~7.7% increase in tokens used (\( \frac{2.24T}{2.08T}
\approx 1.077 \)). Multiplying by the per-million cost, Gemini's
effective expense becomes \[ 3.44 \times 1.077 \approx \$3.70 \] per
million effective tokens—still slightly lower than GPT-4o's $4.38. Yet
in real chained features, early errors often trigger exponential
rework (e.g., 10–20× more tokens on affected branches), easily pushing
Gemini's total cost 30–50% higher overall. Thus, despite Gemini's
lower sticker price, the hidden penalty from extra iterations
frequently makes it the more expensive option in practice for reliable
programming workflows. **And we didn't even mention the lost time.**

> And yes, I wrote the first paragraph of this section and the rest I
> prompted [Grok](https://grok.com) to write. That's the world we live
> in!

# Conclusion

Time did not stand still yet! We are all still swimming up the AI
wave. I am catching speed, so are you. Who will surf down first?

<br>
<br>
