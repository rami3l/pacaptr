<!-- markdownlint-disable MD033 -->

# pacaptr

![pacaptr][socialify badge]

<!--
[pacaptr-logo]: https://user-images.githubusercontent.com/33851577/110216527-e61d7980-7eaf-11eb-9c83-9ab6bccc067a.png
-->

[socialify badge]: https://socialify.git.ci/rami3l/pacaptr/image?description=1&font=Inter&forks=1&issues=1&logo=https%3A%2F%2Fuser-images.githubusercontent.com%2F33851577%2F110216527-e61d7980-7eaf-11eb-9c83-9ab6bccc067a.png&owner=1&pattern=Charlie%20Brown&pulls=1&stargazers=1&theme=Light

`pac·apt·r`, or the _PACman AdaPTeR_, is a wrapper for many package managers with pacman-style command syntax, started as a Rust port of [icy/pacapt].

It's highly recommended to set `pacman` as the alias of `pacaptr` on your non-ArchLinux OS.

Run `pacman -Syu` on the OS of your choice!

```smalltalk
> pacaptr -S neofetch
  Pending `brew reinstall neofetch`
  Proceed [YES/All/No/^C]? y
  Running `brew reinstall neofetch`
==> Downloading https://homebrew.bintray.com/bottles/neofetch-7.1.0
########################################################### 100.0%
==> Reinstalling neofetch
==> Pouring neofetch-7.1.0.big_sur.bottle.tar.gz
🍺  /usr/local/Cellar/neofetch/7.1.0: 6 files, 351.7KB
```

<!--

![Rust Badge](https://img.shields.io/badge/WARNING-Rusty-red?logo=rust&style=flat-square)
![Arch Linux Badge](https://img.shields.io/badge/BTW-I--Use--Arch-blue?logo=arch-linux&style=flat-square)

-->

---

## Contents

- [pacaptr](#pacaptr)
  - [Contents](#contents)
  - [Supported Package Managers](#supported-package-managers)
    - [Windows](#windows)
    - [macOS](#macos)
    - [Linux](#linux)
    - [External](#external)
    - [Notes](#notes)
  - [Installation](#installation)
    - [`brew`](#brew)
    - [`choco`](#choco)
    - [`cargo`](#cargo)
    - [Packaging for `Debian`](#packaging-for-debian)
  - [Configuration](#configuration)
  - [General Tips](#general-tips)
    - [`--using`, `--pm`](#--using---pm)
    - [Automatic `sudo` invocation](#automatic-sudo-invocation)
    - [Extra flags support](#extra-flags-support)
    - [`--dryrun`, `--dry-run`](#--dryrun---dry-run)
    - [`--yes`, `--noconfirm`, `--no-confirm`](#--yes---noconfirm---no-confirm)
    - [`--nocache`, `--no-cache`](#--nocache---no-cache)
  - [Platform-Specific Tips](#platform-specific-tips)
    - [`brew`](#brew-1)
    - [`choco`](#choco-1)
    - [`pip`](#pip)
  - [Postscript](#postscript)

---

## Supported Package Managers

`pacaptr` currently supports the following package managers (in order of precedence):

### Windows

`scoop`, [`choco`](#choco)

### macOS

[`brew`](#brew), `port`

### Linux

`apt`, `apk`, `dnf`, `emerge`, `zypper`

### External

> These are only available with the [`pacaptr --using <name>`](#--using---pm) syntax.

`conda`, `brew`, [`pip`/`pip3`](#pip), `tlmgr`

### Notes

As for now, the precedence is still (unfortunately) hardcoded. For example, if both `scoop` and `choco` are installed, `scoop` will be the default. You can however edit the default package manager in your [config](#configuration).

Please refer to the [compatibility table] for more details on which operations are supported.

Feel free to open a feature/pull request to add support for other package managers :)

## Installation

[We need your help](https://github.com/rami3l/pacaptr/issues/5) to achieve binary distribution of `pacaptr` on more platforms!

### `brew`

```bash
brew install rami3l/tap/pacaptr
```

### `choco`

```powershell
choco install pacaptr
```

### `cargo`

As for now, uploading `pacaptr` to crates.io has been blocked by [cargo/#4468](https://github.com/rust-lang/cargo/issues/4468), so we have to stick with GitHub when building from source.

To install:

```bash
cargo install pacaptr --git https://github.com/rami3l/pacaptr.git
```

To clone and install (for the interested Rustaceans):

```bash
git clone https://github.com/rami3l/pacaptr.git && cd pacaptr
cargo install --path .
```

To uninstall:

```bash
cargo uninstall pacaptr
```

With default settings, the binary should be installed as:

```bash
$HOME/.cargo/bin/pacaptr
```

For `Alpine Linux` users, `cargo build` won't just work, please try this instead:

```bash
RUSTFLAGS="-C target-feature=-crt-static" cargo build
```

### Packaging for `Debian`

```bash
cargo install cargo-deb
cargo deb
```

## Configuration

The default path for the config file is `$HOME/.config/pacaptr/pacaptr.toml`, which can be overridden by the `PACAPTR_CONFIG` environment variable.

I decided not to trash user's `$HOME` without their permission, so:

- If the user hasn't yet specified any path to look at, we will look for the config file in the default path.

- If the config file is not present anyway, a default one will be loaded with `Default::default`, and no files will be written.

<details><summary>Example</summary>

```toml
# This enforces the use of `install` instead of
# `reinstall` in `pacaptr -S`
needed = true

# Explicitly set the default package manager
default_pm = "choco"

# dry_run = false
# no_confirm = false
# no_cache = false
```

</details>

## General Tips

### `--using`, `--pm`

Use this flag to explicitly specify the underlying package manager to be invoked.

```bash
# Here we force the use of `choco`,
# so the following output is platform-independent:
pacaptr --using choco -Su --dryrun
# Canceled: choco upgrade all
```

This can be useful when you are running Linux and you want to use `linuxbrew`, for example. In that case, you can `--using brew`.

### Automatic `sudo` invocation

If you are not `root` and you wish to do something requiring `sudo`, `pacaptr` will do it for you by invoking `sudo -S`.

This feature is currently available for `apk`, `apt`, `dnf`, `emerge`, `port` and `zypper`.

### Extra flags support

The flags after a `--` will be passed directly to the underlying package manager:

```bash
pacaptr -h
# USAGE:
#     pacaptr [FLAGS] [KEYWORDS]... [-- <EXTRA_FLAGS>...]

pacaptr -S curl docker --dryrun -- --proxy=localhost:1234
# Canceled: foo install curl --proxy=localhost:1234
# Canceled: foo install docker --proxy=localhost:1234
```

Here `foo` is the name of your package manager.
(The actual output is platform-specific, which largely depends on if `foo` can actually read the flags given.)

### `--dryrun`, `--dry-run`

Use this flag to just print out the command to be executed
(sometimes with a --dry-run flag to activate the package manager's dryrun option).

`Pending` means that the command execution has been blocked by a prompt; `Canceled` means it has been canceled in a dry run; `Running` means that it has started running.

Some query commands might still be run, but anything "big" should have been stopped from running, eg. installation.
For instance:

```bash
# Nothing will be installed,
# as `brew install curl` won't run:
pacaptr -S curl --dryrun
# Canceled: brew install curl

# Nothing will be deleted here,
# but `brew cleanup --dry-run` is actually running:
pacaptr -Sc --dryrun
# Running: brew cleanup --dry-run
# .. (showing the files to be removed)

# To remove the forementioned files,
# run the command above again without `--dryrun`:
pacaptr -Sc
# Running: brew cleanup
# .. (cleaning up)
```

### `--yes`, `--noconfirm`, `--no-confirm`

Use this flag to trigger the corresponding flag of your package manager (if possible) in order to answer "yes" to every incoming question.

This option is useful when you don't want to be asked during installation, for example, but it can also be dangerous if you don't know what you're doing!

### `--nocache`, `--no-cache`

Use this flag to remove cache after package installation.

This option is useful when you want to reduce `Docker` image size, for example.

## Platform-Specific Tips

### `brew`

- Please note that `cask` is for `macOS` only.

- Be careful when a formula and a cask share the same name, eg. `docker`.

  ```bash
  pacaptr -Si docker | rg cask
  # => Warning: Treating docker as a formula. For the cask, use homebrew/cask/docker

  # Install the formula `docker`
  pacaptr -S docker

  # Install the cask `docker`
  pacaptr -S homebrew/cask/docker

  # Make homebrew treat all keywords as casks
  pacaptr -S docker -- --cask
  ```

- To use `-Rss`, you need to install [rmtree] first:

  ```bash
  brew tap beeftornado/rmtree
  ```

### `choco`

- Don't forget to run in an elevated shell! You can do this easily with tools like [gsudo].

### `pip`

- Use `pacaptr --using pip3` if you want to run the `pip3` command.

## Postscript

Coming from `Arch Linux` to `macOS`, I really like the idea of having an automated version of [Pacman Rosetta] for making common package managing tasks less of a travail thanks to the concise `pacman` syntax.

Initially, I found [icy/pacapt] which does just that, and I made this project to improve `pacapt`'s `homebrew` (especially `cask`) support. (See [pacapt/#117].)

After some discussions in [pacapt/#126], I decided to rewrite the project in Rust to improve readability, testing, etc.

[pacman rosetta]: https://wiki.archlinux.org/index.php/Pacman/Rosetta
[icy/pacapt]: https://github.com/icy/pacapt
[pacapt/#117]: https://github.com/icy/pacapt/issues/117
[pacapt/#126]: https://github.com/icy/pacapt/issues/126
[rmtree]: https://github.com/beeftornado/homebrew-rmtree
[gsudo]: https://github.com/gerardog/gsudo
[rs-dev]: https://github.com/rami3l/pacaptr/tree/rs-dev
[compatibility table]: https://docs.rs/pacaptr
