---
title: Home
date: 2025-12-12
slug: home
---

Welcome
=======

This is a "minimal" blog I created to demonstrate my infrastructure skills, while at the same time getting a personal blog online. Its functionality on the surface is similar to [GitHub Pages](https://docs.github.com/en/pages), I add [Markdown](https://daringfireball.net/projects/markdown) files to it and it gets deployed in my personal domain.

So what's different? Well, I wrote the whole infrastructure implementation. The blog engine is written in [Rust](https://rust-lang.org), and it is running in a [Docker](https://docker.com) image hosted in [Google Cloud](https://cloud.google.com). My code is hosted in [GitHub](https://github.com/pasunboneleve/gcp-rust-blog-public), so every time I push an update, it gets deployed automagically. That's pretty neat.

Why didn't I just use GitHub? Well, this way I can both advertise my skills and have the freedom to improve on this solution. For example, I just included **hot reloading** to the backend. Meaning, when I develop in my laptop, as soon as save a change, my browser displays it. That way I can see what it will look like, and if something is wrong, I can pick it up straight away. Noice.

Enjoy your stay!

### Posts
  <section>
    <ul>
      {{ posts }}
    </ul>
  </section>
