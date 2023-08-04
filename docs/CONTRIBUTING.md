# Contributing to `pacaptr`

Welcome to `pacaptr`!

## Contents

- [Contributing to `pacaptr`](#contributing-to-pacaptr)
  - [Contents](#contents)
  - [WARNING](#warning)
  - [Coding Conventions](#coding-conventions)
  - [API Docs](#api-docs)

## WARNING

This project is still slowly evolving, and the conventions and the APIs could be changed.
Some discussions concerning certain crucial design choices haven't been made yet.

## Coding Conventions

- Rust code: Use `cargo +nightly fmt` and stick with [`rustfmt.toml`](../rustfmt.toml). Follow `cargo clippy` lints if possible.
- Commit message: See [Conventional Commits](https://conventionalcommits.org).

## API Docs

The API docs is a good starting point if you want to dive a little deeper into this project.
You can get it in one of the following ways:

- See the precompiled version on [GitHub Pages](https://rami3l.github.io/pacaptr).
- Compile from source:

  ```bash
  cargo doc --document-private-items --open
  ```
