# Contributing to `pacaptr`

<!-- prettier-ignore -->
> **Warning**
> This project is still slowly evolving, and the conventions and the APIs could be changed.
> Some discussions concerning certain crucial design choices haven't been made yet.

Welcome to `pacaptr`!

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

## Making a New Release

We currently make a new release by pushing a single new version tag to `master`, which will make the CI generate a new GitHub release together with the necessary artifacts.

To make this automatic (and to push the new version to crates.io at the same time), it is recommended to use [`cargo-release`](https://github.com/crate-ci/cargo-release):

- Perform a dry run to see if everything is OK[^patch]:

  ```bash
  cargo release --workspace patch
  ```

  [^patch]:
      This example uses `patch` (0.0.1).
      Depending on the situation, `minor` (0.1) or `major` (1.0) might be used instead.

- Add `-x` to actually publish the new version.
