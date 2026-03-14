---
title: "The responsibility trap"
date: 2026-03-13
slug: 2026-03-13-the-responsibility-trap
description: "Two stories about fast engineers, slow engineers, and what responsibility does to both."
image: "/static/responsibility-trap-liotard-social.jpg"
---

Anyone who has worked in software teams has seen it. One engineer ships
features constantly, while another seems to take much longer to make
progress.

---

## The slow engineer

In one team I joined, there was an engineer who knew the system better
than anyone else. Let’s call him Bob.

The platform we worked on was complicated and difficult to reason
about. It had accumulated layers of architecture, historical
constraints, and operational quirks that were not obvious to
newcomers. Bob understood all of it. He was usually the first person
asked to implement new features.

But he moved slowly.

From the outside the reason seemed clear. The system was fragile, and
Bob had learned to move carefully inside it. At the time I thought the
real problem was simpler: the architecture itself. To me the system
had become too complex, full of implicit constraints and historical
decisions baked into it.

The right answer was to simplify it.

So I proposed something ambitious: a large architectural change that
would remove many of those layers and make the system easier to reason
about.

Bob left the team not long after.\
The changes went ahead.

<figure>
  <a href="https://commons.wikimedia.org/wiki/File:A_Hungarian_nobleman_and_peasant_Townson_cropped.jpg" target="_blank" rel="noopener noreferrer" style="text-decoration:none;border:0;display:block;max-width:720px;width:100%;margin:0.5rem auto;">
    <img class="block w-full sm:w-3/5 h-auto mx-auto" src="https://upload.wikimedia.org/wikipedia/commons/8/89/A_Hungarian_nobleman_and_peasant_Townson_cropped.jpg" alt="A Hungarian nobleman and peasant" loading="lazy" style="background:transparent;" />
  </a>
  <figcaption class="w-full sm:w-3/5 mx-auto text-center" style="margin-top:0.5rem;">
    <strong>Figure 1.</strong> <em>A Hungarian nobleman and peasant</em>, from Robert Townson.
    Source: <a href="https://commons.wikimedia.org/wiki/File:A_Hungarian_nobleman_and_peasant_Townson_cropped.jpg" target="_blank" rel="noopener noreferrer">Wikimedia Commons</a>.
  </figcaption>
</figure>

---

## The fast engineer

Some time later I joined another team. Over years, I added significant
improvements to the codebase we worked on. Eventually a new engineer
joined the team. Let’s call her Alice.

Alice was extraordinarily productive. Features appeared quickly. Every
sprint seemed to produce visible progress from her. She also proposed
architectural improvements that were thoughtful and well
reasoned. Many of them made the codebase simpler, stabler, and faster.

I could not keep up.

I was responsible for parts of the system that required constant
attention: infrastructure that was not a business priority, small edge
cases, and maintaining the flagship offering. From the outside the
comparison again looked simple.

Alice was shipping features.\
I wasn't.

Eventually the company concluded Alice was the better investment.\
I was made redundant.

---

## Looking back

In the first story:

- Bob knew the system deeply
- To me the system was complicated and difficult to reason about
- He moved cautiously inside it
- I arrived proposing large changes

In the second story:

- I knew the system deeply
- To Alice the system was complicated and difficult to reason about
- I moved cautiously inside it
- Alice arrived proposing large changes

The roles had reversed.

---

## The responsibility trap

The difference was not the engineers.\
The difference was responsibility.

The person responsible for keeping a system stable must absorb its
complexity. They see the operational constraints, the historical
trade-offs, and the subtle ways a change can break something else.

The newcomer sees something different. They see the architecture from
the outside. They see opportunities for change that the person
responsible for stability must treat cautiously.

Both perspectives are real. But they produce very different apparent speeds.

Responsibility trades speed for stability.

---

## A systems perspective

This pattern shows up often in complex systems. The cybernetician W.
Ross Ashby formulated the _Law of Requisite Variety_: a system must
have enough internal flexibility to absorb disturbances from its
environment.

In software systems, the engineers responsible for stability often
become the regulators of that system. They absorb complexity and
disturbances so the rest of the organisation can continue operating.

But regulation consumes attention.

As systems grow more complex, the engineers performing this role
naturally move more cautiously. From the outside this can look like
declining productivity.

---

## No villains

In both stories it would be easy to assign blame. One could accuse the
slow engineer of moving too cautiously. One could accuse management of
rewarding visible progress rather than invisible stability.

But that interpretation misses the deeper pattern.

No one was behaving irrationally.

The organisation was responding to the signals it could see. And those
signals were shaped by the roles people were playing inside the
system.

---

## Why platforms exist

This is one reason platform engineering exists. Platforms reduce the
regulatory burden placed on individual engineers. By standardising
infrastructure, automating operational work, and providing paved
roads, they reduce the number of disturbances that must be absorbed
manually.

When that burden decreases, engineers regain the freedom to focus on
change.

Speed returns, not because the engineers became better, but because
the system became easier to regulate.

---

<figure>
  <a href="https://commons.wikimedia.org/wiki/File:Jean-Etienne_Liotard_-_A_Lady_in_Turkish_Dress_and_Her_Servant_-_Google_Art_ProjectFXD.jpg" target="_blank" rel="noopener noreferrer" style="text-decoration:none;border:0;display:block;max-width:720px;width:100%;margin:0.5rem auto;">
    <img class="block w-full sm:w-3/5 h-auto mx-auto" src="https://upload.wikimedia.org/wikipedia/commons/7/7c/Jean-Etienne_Liotard_-_A_Lady_in_Turkish_Dress_and_Her_Servant_-_Google_Art_ProjectFXD.jpg" alt="A Lady in Turkish Dress and Her Servant, by Jean-Etienne Liotard" loading="lazy" style="background:transparent;" />
  </a>
  <figcaption class="w-full sm:w-3/5 mx-auto text-center" style="margin-top:0.5rem;">
    <strong>Figure 2.</strong> <em>A Lady in Turkish Dress and Her Servant</em>, Jean-Etienne Liotard.
    Source: <a href="https://commons.wikimedia.org/wiki/File:Jean-Etienne_Liotard_-_A_Lady_in_Turkish_Dress_and_Her_Servant_-_Google_Art_ProjectFXD.jpg" target="_blank" rel="noopener noreferrer">Wikimedia Commons</a> (public domain).
  </figcaption>
</figure>
