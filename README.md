<!-- markdownlint-disable MD033 -->

# pacaptr

[![pacaptr][socialify badge]](https://rami3l.github.io/pacaptr/pacaptr/)

<!--
[pacaptr-logo]: https://user-images.githubusercontent.com/33851577/110216527-e61d7980-7eaf-11eb-9c83-9ab6bccc067a.png
-->

[socialify badge]: https://socialify.git.ci/rami3l/pacaptr/image?description=1&font=Inter&logo=https%3A%2F%2Fuser-images.githubusercontent.com%2F33851577%2F110216527-e61d7980-7eaf-11eb-9c83-9ab6bccc067a.png&owner=1&pattern=Charlie%20Brown&theme=Light

<!--
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat-square&logo=rust&logoColor=white)
-->

[![Crates.io](https://img.shields.io/crates/v/pacaptr?style=flat-square)](https://crates.io/crates/pacaptr)
[![docs.rs](https://img.shields.io/docsrs/pacaptr?style=flat-square)](https://docs.rs/pacaptr)
[![License](https://img.shields.io/github/license/rami3l/pacaptr?style=flat-square)](LICENSE)

`pac·apt·r`, or the _PACman AdaPTeR_, is a wrapper for many package managers with pacman-style command syntax, started as a Rust port of [icy/pacapt].

Just set `pacman` as the alias of `pacaptr` on your non-ArchLinux OS, and you can run `pacman -Syu` wherever you like!

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
  - [Why `pacaptr`?](#why-pacaptr)
  - [Supported Package Managers](#supported-package-managers)
  - [Installation](#installation)
    - [`brew`](#brew)
    - [`choco`](#choco)
    - [`cargo`](#cargo)
    - [Packaging for `Debian`](#packaging-for-debian)
  - [Configuration](#configuration)
  - [Tips](#tips)
    - [Universal](#universal)
      - [`--using`, `--pm`](#--using---pm)
      - [Automatic `sudo` invocation](#automatic-sudo-invocation)
      - [Extra flags support](#extra-flags-support)
      - [`--dryrun`, `--dry-run`](#--dryrun---dry-run)
      - [`--yes`, `--noconfirm`, `--no-confirm`](#--yes---noconfirm---no-confirm)
      - [`--nocache`, `--no-cache`](#--nocache---no-cache)
    - [Platform-Specific Tips](#platform-specific-tips)
      - [For `brew`](#for-brew)
      - [For `choco`](#for-choco)
      - [For `pip`](#for-pip)
      - [For `scoop`](#for-scoop)
    - [Feel Like Contributing?](#feel-like-contributing)

---

## Why `pacaptr`?

Coming from `Arch Linux` to `macOS`, I really like the idea of having an automated version of [Pacman Rosetta] for making common package managing tasks less of a travail thanks to the concise `pacman` syntax.

That's why I decided to take inspiration from the existing `sh`-based project [icy/pacapt] to make a new CLI tool in Rust for better portability (especially for Windows) and easier maintenance.

## Supported Package Managers

`pacaptr` currently supports the following package managers (in order of precedence):

- Windows: `scoop`, [`choco`](#choco)
- macOS: [`brew`](#brew), `port`, `apt` (through [Procursus])
- Linux: `apt`, `apk`, `dnf`, `emerge`, `xbps`, `zypper`
- External: `brew`, `conda`, [`pip`/`pip3`](#pip), `tlmgr`
  - These are only available with the [`pacaptr --using <name>`](#--using---pm) syntax.

As for now, the precedence is still (unfortunately) hardcoded. For example, if both `scoop` and `choco` are installed, `scoop` will be the default. You can however edit the default package manager in your [config](#configuration).

Please refer to the [compatibility table] for more details on which operations are supported.

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

To install the release version from crates.io:

```bash
cargo install pacaptr
```

To install `master` version from GitHub:

```bash
cargo install pacaptr --git https://github.com/rami3l/pacaptr.git
```

To clone and install (for the interested Rustaceans):

```bash
git clone https://github.com/rami3l/pacaptr.git && cd pacaptr
cargo install --path . # Output path is usually `$HOME/.cargo/bin/pacaptr`.
```

To uninstall:

```bash
cargo uninstall pacaptr
```

For `Alpine Linux` users, `cargo build` might not just work, in this case, please try the following instead:

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

## Tips

### Universal

#### `--using`, `--pm`

Use this flag to explicitly specify the underlying package manager to be invoked.

```bash
# Here we force the use of `choco`,
# so the following output is platform-independent:
pacaptr --using choco -Su --dryrun
# Canceled: choco upgrade all
```

This can be useful when you are running Linux and you want to use `linuxbrew`, for example. In that case, you can `--using brew`.

#### Automatic `sudo` invocation

If you are not `root` and you wish to do something requiring `sudo`, `pacaptr` will do it for you by invoking `sudo -S`.

This feature is currently available for `apk`, `apt`, `dnf`, `emerge`, `port` and `zypper`.

#### Extra flags support

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

#### `--dryrun`, `--dry-run`

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

#### `--yes`, `--noconfirm`, `--no-confirm`

Use this flag to trigger the corresponding flag of your package manager (if possible) in order to answer "yes" to every incoming question.

This option is useful when you don't want to be asked during installation, for example, but it can also be dangerous if you don't know what you're doing!

#### `--nocache`, `--no-cache`

Use this flag to remove cache after package installation.

This option is useful when you want to reduce `Docker` image size, for example.

### Platform-Specific Tips

#### For `brew`

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

#### For `choco`

- Don't forget to run in an elevated shell! You can do this easily with tools like [gsudo].

#### For `pip`

- Use `pacaptr --using pip3` if you want to run the `pip3` command.

#### For `scoop`

- `pacaptr` launches a `powershell` subprocess to run `scoop`, so please make sure that you have set the right execution policy in `powershell` ([**not `pwsh`**](https://powershellexplained.com/2017-12-29-Powershell-what-is-pwsh/)):

  ```pwsh
  Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
  ```

### Feel Like Contributing?

Sounds nice! Please let me take you to the [contributing guidelines](docs/CONTRIBUTING.md) :)

[pacman rosetta]: https://wiki.archlinux.org/index.php/Pacman/Rosetta
[icy/pacapt]: https://github.com/icy/pacapt
[pacapt/#117]: https://github.com/icy/pacapt/issues/117
[pacapt/#126]: https://github.com/icy/pacapt/issues/126
[rmtree]: https://github.com/beeftornado/homebrew-rmtree
[gsudo]: https://github.com/gerardog/gsudo
[rs-dev]: https://github.com/rami3l/pacaptr/tree/rs-dev
[compatibility table]: https://rami3l.github.io/pacaptr/pacaptr/#compatibility-table
[procursus]: https://github.com/ProcursusTeam/Procursus
