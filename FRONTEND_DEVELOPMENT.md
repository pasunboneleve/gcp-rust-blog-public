Frontend Development
--------------------

Quickstart
==========

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
./scripts/tailwatch.sh
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
