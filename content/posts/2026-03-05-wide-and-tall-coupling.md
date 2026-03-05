---
title: "Wide and tall coupling"
date: 2026-03-05
slug: 2026-03-05-wide-and-tall-coupling
image: "/static/wide-tall-coupling-social.jpg"
---

Fast development often backfires. Engineering cultures that only value
rapid delivery can produce systems that work well today but become
expensive to change. When this happens, coupling tends to grow in two
directions:

- **wide coupling** — spreading across components
- **tall coupling** — growing inside a component

## Individual speed vs organisational speed

Before going further, it helps to clarify what we mean by
*velocity*. This tension has been noted by Charity Majors:

> The fact that individual productivity does not necessarily translate
> into organizational velocity — can in fact impede organizational
> velocity — is real and under-addressed.

For an individual engineer, velocity usually means output:

- implementing functionality
- fixing bugs
- improving stability

For the organisation, velocity means something else: **the ability to
change when reality changes.** Markets move, requirements shift, and
whole product assumptions expire. A system that cannot adapt quickly
becomes a liability, no matter how polished it once looked.

So we are dealing with two optimisation targets.

- **Individual velocity**: producing more code and features.

- **Organisational velocity**: adapting the system when assumptions
  change.

These goals are not always aligned.

## Wide coupling

<p style="text-align: center;"> <a
  href="https://upload.wikimedia.org/wikipedia/commons/e/e5/1845_Logerot_Jigsaw_Puzzle_Atlas_of_the_World_-_Geographicus_-_Atlas-logerot-1845.jpg"
  target="_blank" rel="noopener noreferrer"
  style="text-decoration:none;border:0;"> <img
  src="/static/longerot-1845-geographical-puzzle-mid-yellow.jpg"
  alt="1845 Logerot jigsaw puzzle atlas of the world" loading="lazy"
  style="display:block;margin:0.5rem
  auto;max-width:720px;width:100%;height:auto;background:transparent;"
  /> </a> </p> <p style="font-size: smaller; color: #93a1a1;
  text-align: center;"><strong>Figure 1.</strong> 1845 Logerot jigsaw
  puzzle atlas of the world. Source: <a
  href="https://upload.wikimedia.org/wikipedia/commons/e/e5/1845_Logerot_Jigsaw_Puzzle_Atlas_of_the_World_-_Geographicus_-_Atlas-logerot-1845.jpg"
  target="_blank" rel="noopener noreferrer">Wikimedia Commons</a>
  (public domain).</p>

Wide coupling is the more familiar form. The [Unix
philosophy](http://www.catb.org/~esr/writings/taoup/html/ch01s06.html)
captures the opposite ideal well:

> Make each program do one thing well.

Consider `grep`. It reads text streams and finds matches. Because it
works on plain streams, it composes cleanly with other tools and can
be replaced without rewriting everything around it.

A text editor search feature is different. Replacing it usually
requires understanding internal data structures, UI wiring, and
surrounding logic. The capability is embedded across multiple pieces.

That is wide coupling.

My mental model is a puzzle with glue spilled across adjacent
pieces. The glue hardens sideways, so removing one piece means
disturbing several others. Change that should be local becomes
systemic. Local change becomes difficult.

I once worked on decoupling a multi-service application built around a
shared database. Every service read and wrote directly to the same
schema. We moved business logic out of the database and shifted
service communication to HTTP and queues. Performance and clarity
improved, but a new coupling appeared. The
[Avro](https://avro.apache.org/) schemas between services mirrored the
database schemas closely. When one schema changed, several services
had to change with it. Local change was still difficult.

The glue had moved from the database to the message formats.

## Tall coupling

Tall coupling is different. Instead of spreading sideways, complexity
stacks upward inside a single component. Boundaries remain intact, but
the component gets deeper and denser over time. More responsibilities
accumulate. More formats are supported. More behaviour is
embedded. Eventually only a few people understand the component well
enough to modify it safely.

Replacing it would require enormous effort.\
So nobody touches it.

Imagine writing a replacement for `grep`. It starts as text search,
then grows to handle JSON, YAML, and Parquet, and eventually becomes
the universal query engine for the organisation’s Lakehouse. At that
point it may be deeply useful and deeply risky at the same time.

Change slows down.\
The component did not grow sideways.\
It grew **tall**.

<p style="text-align: center;"> <a
  href="https://commons.wikimedia.org/wiki/File:Flatiron_Building_under_construction_II,_New_York_City,_1902.jpg"
  target="_blank" rel="noopener noreferrer"
  style="text-decoration:none;border:0;"> <span
  style="position:relative;display:block;max-width:504px;width:100%;margin:0.5rem
  auto;"> <img
  src="https://upload.wikimedia.org/wikipedia/commons/6/6e/Flatiron_Building_under_construction_II%2C_New_York_City%2C_1902.jpg"
  alt="Flatiron Building under construction II, New York City, 1902"
  loading="lazy"
  style="display:block;width:100%;height:auto;background:transparent;opacity:1;"
  /> <span style="position:absolute;inset:0;background:rgb(128, 106,
  56);opacity:0.40;pointer-events:none;"></span> </span> </a> </p> <p
  style="font-size: smaller; color: #93a1a1; text-align:
  center;"><strong>Figure 2.</strong> Flatiron Building under
  construction II, New York City, 1902. Source: <a
  href="https://commons.wikimedia.org/wiki/File:Flatiron_Building_under_construction_II,_New_York_City,_1902.jpg"
  target="_blank" rel="noopener noreferrer">Wikimedia Commons</a>
  (public domain).</p>

## Why this matters

Both wide and tall coupling reduce organisational velocity. Wide
coupling spreads change across many components; tall coupling
concentrates complexity inside one component.

In simple terms:

- **Wide coupling** change spreads across many components.
- **Tall coupling** change is trapped inside one massive component.

In both cases the cost of change rises. The system becomes harder to
reshape when the business needs it to move.

If this sounds familiar, it is because coupling directly affects
something more fundamental: the **cost of change**. Wide coupling
increases the number of components that must change together. Tall
coupling increases the amount of complexity that must be understood
before making a change.

In both cases the price of modification rises, and organisational
velocity falls.

## Keeping coupling in check

Coupling is difficult to eliminate entirely, especially in data-heavy
systems where highly normalised models introduce natural
dependencies. There is no universal fix. What helps is awareness and
boundary discipline. The **Unix principle** remains a strong starting
point:

> Make each program do one thing well.

The longevity of a system depends on intentionality.

“Just start typing” is a reliable way to type yourself into a corner.

Both tall and wide coupling appear quickly when boundaries are not
actively maintained.

Work starts with conceptual clarity:

- each component should do one thing well
- responsibilities should be visible
- components should be replaceable

Shallow components resist tall coupling.\
Narrow interfaces resist wide coupling.

APIs should also be easy to test. Mocking is a simple boundary health
check: if mocking an API is painful, the interface is probably doing
too much.

## Monoliths, microservices, and change

A common observation is that monoliths are not inherently bad; messy
microservices often just move complexity from internal layers to
external ones. That is true. Still, decomposition has one useful side
effect: it makes complexity visible. Once visible, teams can ask
better questions.

Do we actually need this complexity?\
Is this boundary meaningful?\
Can this component be replaced?

These conversations are harder when dependencies remain buried in one
codebase. Ultimately, architecture style matters less than one
property: **how cheap it is to change the system.**

Wide coupling spreads change across the system.\
Tall coupling buries
change inside complexity.\
Both slow the organisation down.

Because organisational velocity is not measured by how fast we write
code. It is measured by how quickly we can reshape the system when
reality changes.

<p style="text-align: center;"> <a
  href="https://commons.wikimedia.org/wiki/File:348Sandbild_im_Sera_Kloster.jpg"
  target="_blank" rel="noopener noreferrer"
  style="text-decoration:none;border:0;"> <span
  style="position:relative;display:block;max-width:432px;width:100%;margin:0.5rem
  auto;"> <img
  src="https://upload.wikimedia.org/wikipedia/commons/7/74/348Sandbild_im_Sera_Kloster.jpg"
  alt="Sand mandala at Sera Monastery" loading="lazy"
  style="display:block;width:100%;height:auto;background:transparent;opacity:1;"
  /> <span style="position:absolute;inset:0;background:rgb(128, 106,
  56);opacity:0.20;pointer-events:none;"></span> </span> </a> </p> <p
  style="font-size: smaller; color: #93a1a1; text-align:
  center;"><strong>Figure 3.</strong> Sand mandala at Sera
  Monastery. Sand mandalas are made with extreme precision, then
  ceremonially dismantled and dispersed, often into water. The
  practice teaches impermanence: even the most beautiful, complex
  structures are temporary. It is both construction and release,
  demanding care in creation and non-attachment in letting go. Source:
  <a
  href="https://commons.wikimedia.org/wiki/File:348Sandbild_im_Sera_Kloster.jpg"
  target="_blank" rel="noopener noreferrer">Wikimedia Commons</a> (CC
  BY 3.0).</p>

---

<p style="font-size: smaller; color: #93a1a1; line-height:
1.5;"><sup>Note.</sup> I explored this idea previously when discussing
the <a href="/posts/2026-03-04-cost-of-change">cost of change in a
data platform</a>.</p>
