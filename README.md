<!-- markdownlint-disable MD033 -->

# pacaptr

[![pacaptr](https://socialify.git.ci/rami3l/pacaptr/image?description=1&font=Inter&logo=https%3A%2F%2Fgithub.com%2Frami3l%2Fpacaptr%2Fblob%2Fmaster%2Fassets%2Flogo.png%3Fraw%3Dtrue&name=1&owner=1&pattern=Solid&theme=Light)](https://crates.io/crates/pacaptr)

<!--
[pacaptr-logo]: https://github.com/rami3l/pacaptr/blob/master/assets/logo.png?raw=true
-->

<!--
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat-square&logo=rust&logoColor=white)
-->

[![Crates.io](https://img.shields.io/crates/v/pacaptr?style=flat-square)](https://crates.io/crates/pacaptr)
[![docs.rs](https://img.shields.io/docsrs/pacaptr?style=flat-square)](https://docs.rs/pacaptr)
[![Private APIs](https://img.shields.io/badge/docs-private--apis-lightgrey?style=flat-square)](https://rami3l.github.io/pacaptr)
[![License](https://img.shields.io/github/license/rami3l/pacaptr?style=flat-square)](LICENSE)

`pac·apt·r`, or the _PACman AdaPTeR_, is a wrapper for many package managers that allows you to use pacman commands with them.

Just set `pacman` as the alias of `pacaptr` on your non-Arch OS, and then you can run `pacman -Syu` wherever you like!

```smalltalk
> pacaptr -S neofetch
  Pending `brew reinstall neofetch`
  Proceed with the previous command? · Yes
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

## Why `pacaptr`?

Coming from `Arch Linux` to `macOS`, I really like the idea of having an automated version of [Pacman Rosetta] for making common package managing tasks less of a travail thanks to the concise `pacman` syntax.

That's why I decided to take inspiration from the existing `sh`-based [icy/pacapt] to make a new CLI tool in Rust for better portability (especially for Windows and macOS) and easier maintenance.

## Supported Package Managers

`pacaptr` currently supports the following package managers (in order of precedence):

- Windows
  - [`scoop`](#for-scoop)
  - [`choco`](#for-choco)
  - `winget`
- macOS
  - [`brew`](#for-brew)
  - `port`
  - `apt` (through [Procursus])
- Linux
  - `apt`
  - `apk`
  - `dnf`
  - `emerge`
  - `xbps`
  - `zypper`
- External: These are only available with the [`pacaptr --using <name>`](#--using---pm) syntax.
  - `brew`
  - `conda`
  - [`pip`](#for-pip)/[`pip3`](#for-pip)
  - `pkcon`
  - `tlmgr`

As for now, the precedence is still (unfortunately) hard-coded. For example, if both `scoop` and `choco` are installed, `scoop` will be the default. You can, however, edit the default package manager in your [config](#configuration).

Please refer to the [compatibility table] for more details on which operations are supported.

## Installation

<!-- prettier-ignore -->
> **Note**
> [We need your help](https://github.com/rami3l/pacaptr/issues/5) to achieve binary distribution of `pacaptr` on more platforms!

### Brew

[![Tap Updated](https://img.shields.io/github/last-commit/rami3l/homebrew-tap/master?style=flat-square&label=tap%20updated)](https://github.com/rami3l/homebrew-tap)

```bash
brew install rami3l/tap/pacaptr
```

### Scoop

[![Scoop Version](https://img.shields.io/scoop/v/pacaptr?bucket=extras&style=flat-square)](https://scoop.sh/#/apps?q=pacaptr&o=true)

```powershell
scoop bucket add extras
scoop install pacaptr
```

### Choco

[![Chocolatey Version](https://img.shields.io/chocolatey/v/pacaptr?style=flat-square)](https://community.chocolatey.org/packages/pacaptr)
[![Chocolatey Downloads](https://img.shields.io/chocolatey/dt/pacaptr?style=flat-square)](https://community.chocolatey.org/packages/pacaptr)

```powershell
choco install pacaptr
```

### Cargo

[![Cargo Version](https://img.shields.io/crates/v/pacaptr?style=flat-square)](https://crates.io/crates/pacaptr)
[![Cargo Downloads](https://img.shields.io/crates/d/pacaptr?style=flat-square)](https://crates.io/crates/pacaptr)

If you have installed [`cargo-binstall`], the fastest way of installing `pacaptr` via `cargo` is by running:

```bash
cargo binstall pacaptr
```

To build and install the release version from crates.io:

```bash
cargo install pacaptr
```

To build and install the `master` version from GitHub:

```bash
cargo install pacaptr --git https://github.com/rami3l/pacaptr.git
```

For those who are interested, it is also possible to build and install from your local repo:

```bash
git clone https://github.com/rami3l/pacaptr.git && cd pacaptr
cargo install --path .
# The output path is usually `$HOME/.cargo/bin/pacaptr`.
```

To uninstall:

```bash
cargo uninstall pacaptr
```

For `Alpine Linux` users, `cargo build` might not work. Please try the following instead:

```bash
RUSTFLAGS="-C target-feature=-crt-static" cargo build
```

### Packaging for Debian

```bash
cargo install cargo-deb
cargo deb
```

## Configuration

The config file path is defined with the following precedence:

- `$PACAPTR_CONFIG`, if it is set;
- `$XDG_CONFIG_HOME/pacaptr/pacaptr.toml`, if `$XDG_CONFIG_HOME` is set;
- `$HOME/.config/pacaptr/pacaptr.toml`.

I decided not to trash user's `$HOME` without their permission, so:

- If the user hasn't yet specified any path to look at, we will look for the config file in the default path.

- If the config file is not present anyway, a default one will be loaded with `Default::default`, and no files will be written.

- Any config item can be overridden by the corresponding `PACAPTR_*` environment variable. For example, `PACAPTR_NEEDED=false` is prioritized over `needed = true` in `pacaptr.toml`.

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

This feature is currently available for `apk`, `apt`, `dnf`, `emerge`, `pkcon`, `port`, `xbps` and `zypper`.

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

Some query commands might still be run, but anything "big" should have been stopped from running, e.g. installation.
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

- Be careful when a formula and a cask share the same name, e.g. `docker`.

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

#### For `scoop`

- `pacaptr` launches a [`pwsh`](https://powershellexplained.com/2017-12-29-Powershell-what-is-pwsh/) subprocess to run `scoop`, or a `powershell` one if `pwsh` is not found in `$PATH`. Please make sure that you have set the right execution policy in the corresponding shell:

  ```pwsh
  Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
  ```

#### For `choco`

- Don't forget to run in an elevated shell! You can do this easily with tools like [gsudo].

#### For `pip`

- Use `pacaptr --using pip3` if you want to run the `pip3` command.

### Feel Like Contributing?

Sounds nice! Please let me take you to the [contributing guidelines](docs/CONTRIBUTING.md) :)

[`cargo-binstall`]: https://github.com/cargo-bins/cargo-binstall
[compatibility table]: https://rami3l.github.io/pacaptr/pacaptr/#compatibility-table
[gsudo]: https://github.com/gerardog/gsudo
[icy/pacapt]: https://github.com/icy/pacapt
[pacman rosetta]: https://wiki.archlinux.org/index.php/Pacman/Rosetta
[procursus]: https://github.com/ProcursusTeam/Procursus
