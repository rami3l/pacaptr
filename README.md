# pacaptr

![pacaptr][Socialify Badge]

<!-- ![Rust Badge](https://img.shields.io/badge/WARNING-Rusty-red?logo=rust&style=flat-square) -->
<!-- ![Arch Linux Badge](https://img.shields.io/badge/BTW-I--Use--Arch-blue?logo=arch-linux&style=flat-square) -->
![Interface Concept]

`pacaptr` is a Rust port of [icy/pacapt], a wrapper for many package managers with pacman-style command syntax.

Run `pacaptr -Syu` on the distro of your choice!

## Contents

- [pacaptr](#pacaptr)
  - [Contents](#contents)
  - [Supported Package Managers](#supported-package-managers)
  - [Motivation & Current Status](#motivation--current-status)
  - [Building & Installation](#building--installation)
  - [Configuration](#configuration)
  - [General Tips](#general-tips)
  - [Platform-Specific Tips](#platform-specific-tips)

## Supported Package Managers

- `Windows/chocolatey`
- `macOS/homebrew` (with [auto `cask` invocation](#platform-specific-tips))
- `macOS/macports`
- `Debian/apt`
- `Alpine/apk`
- `RedHat/dnf`
- `SUSE/zypper`
- `External/conda`¹
- `External/linuxbrew`¹
- `External/pip`¹
- `External/tlmgr`¹

¹: Require `pacaptr --using <name>` to invocate (see [general tips](#general-tips)).

Notes:

- Support for more package managers will be added Soon™.
- Don't miss the [general](#general-tips) & [platform-specific](#platform-specific-tips) tips below!

## Motivation & Current Status

Coming from `Arch Linux` to `macOS`, I really like the idea of having an automated version of [Pacman Rosetta] for making common package managing tasks less of a travail thanks to the concise `pacman` syntax.

Initially, I found [icy/pacapt] which does just that, and I made this project to improve `pacapt`'s `homebrew` (especially `cask`) support. (See [pacapt/#117].)

After some discussions in [pacapt/#126], I decided to rewrite the project in Rust to improve readability, testing, etc.

## Building & Installation

PPAs might be added when appropriate.

- `macOS/homebrew` & `External/linuxbrew` install:

  ```bash
  # Short version:
  brew install rami3l/tap/pacaptr

  # Which is equivalent to this:
  brew tap rami3l/tap
  brew install pacaptr
  ```

- `Windows/chocolatey` install:
  
  ```powershell
  # Yes, now we are still in the prerelease stage...
  choco install pacaptr --pre
  ```

- Build from source:

  ```bash
  # First you'll need to download the source:
  git clone https://github.com/rami3l/pacaptr.git
  cd pacaptr

  # To run:
  cargo run -- -S curl

  # To install:
  cargo install --path .

  # To uninstall:
  cargo uninstall pacaptr
  ```

- Packaging for Debian:

  ```bash
  cargo install cargo-deb
  cargo deb
  ```

Notes:

- For `Alpine/apk` users: `cargo build` won't just work, please try this instead:
  
  ```bash
  RUSTFLAGS="-C target-feature=-crt-static" cargo build
  ```

## Configuration

The configuration file is `$HOME/.config/pacaptr/pacaptr.toml`.

An example:

```toml
# This enforces the use of `install` instead of
# `reinstall` in `pacaptr -S`
needed = true

# Explicitly set the default package manager
default_pm = "choco"

# dry_run = false
# no_confirm = false
# force_cask = false
# no_cache = false
```

## General Tips

- `--using`, `--pm`: Use this flag to explicitly specify the underlying package manager to be invoked.

  ```bash
  # Here we force the use of `choco`,
  # so the following output is platform-independent:
  pacaptr --using choco -Su --dryrun
  # Pending: choco upgrade all
  ```

  This can be useful when you are running Linux and you want to use `linuxbrew`, for example. In that case, you can `--using brew`.

- Extra flags support:
  - The flags after a `--` will be passed directly to the underlying package manager:

    ```bash
    pacaptr -h
    # USAGE:
    #     pacaptr [FLAGS] [KEYWORDS]... [-- <EXTRA_FLAGS>...]

    pacaptr -S curl docker --dryrun -- --proxy=localhost:1234
    # Pending: foo install curl --proxy=localhost:1234
    # Pending: foo install docker --proxy=localhost:1234
    ```

    Here `foo` is the name of your package manager.
    (The actual output is platform-specific, which largely depends on if `foo` can actually read the flags given.)

- `--dryrun`, `--dry-run`: Use this flag to just print out the command to be executed
  (sometimes with a --dry-run flag to activate the package manager's dryrun option).

  - `Pending` means that the command execution is blocked (a dry run or prompted to continue),
  while `Running` means that it is running.

  - Some query commands might still be run, but anything "big" should have been stopped from running, eg. installation.
    For instance:

    ```bash
    # Nothing will be installed,
    # as `brew install curl` won't run:
    pacaptr -S curl --dryrun
    # Pending: brew install curl

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

- `--yes`, `--noconfirm`, `--no-confirm`:
  Use this flag to trigger the corresponding flag of your package manager (if possible) in order to answer "yes" to every incoming question.
  - This option is useful when you don't want to be asked during installation, for example.
  - ... But it can be potentially dangerous if you don't know what you're doing!

- `--nocache`, `--no-cache`:
  Use this flag to remove cache after package installation.
  - This option is useful when you want to reduce `Docker` image size, for example.

## Platform-Specific Tips

- `macOS/homebrew` & `External/linuxbrew` support: Please note that `cask` is for macOS only.

  - Automatic `brew cask` invocation: implemented for `-S`, `-R`, `-Su`, and more.

    ```bash
    pacaptr -S curl --dryrun
    # Pending: brew install curl

    pacaptr -S gimp --dryrun
    # Pending: brew cask install gimp
    ```

  - The use of `brew cask` commands can also be enforced by adding a `--cask` flag. Useful when a bottle and a cask share the same name, eg. `docker`.

  - To use `-Rss`, you need to install [rmtree] first:

    ```bash
    brew tap beeftornado/rmtree
    ```

- `Windows/chocolatey` support: Don't forget to run in an elevated shell! You can do this easily with tools like [gsudo].

- `External/pip` support: Use `pacaptr --using pip3` if you want to run the `pip3` command.

[Socialify Badge]: https://socialify.git.ci/rami3l/pacaptr/image?description=1&font=Inter&forks=1&issues=1&logo=https%3A%2F%2Fupload.wikimedia.org%2Fwikipedia%2Fcommons%2Fthumb%2Fd%2Fd5%2FRust_programming_language_black_logo.svg%2F1200px-Rust_programming_language_black_logo.svg.png&owner=1&pattern=Circuit%20Board&pulls=1&stargazers=1&theme=Light
[Interface Concept]: https://carbon.now.sh/?bg=rgba(255%2C255%2C255%2C1)&t=vscode&wt=none&l=smalltalk&ds=false&dsyoff=20px&dsblur=68px&wc=true&wa=true&pv=30px&ph=48px&ln=false&fl=1&fm=JetBrains%20Mono&fs=13.5px&lh=133%25&si=false&es=2x&wm=false&code=%253E%2520pacaptr%2520-S%2520neofetch%250A%2520%2520Pending%2520%2560brew%2520reinstall%2520neofetch%2560%250A%2520%2520Proceed%2520%255BYes%252Fall%252Fno%255D%253F%2520y%250A%2520%2520Running%2520%2560brew%2520reinstall%2520neofetch%2560%250A%253D%253D%253E%2520Downloading%2520https%253A%252F%252Fhomebrew.bintray.com%252Fbottles%252Fneofetch-7.1.0.catalina.bot%250A%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2523%2520100.0%2525%250A%253D%253D%253E%2520Reinstalling%2520neofetch%250A%253D%253D%253E%2520Pouring%2520neofetch-7.1.0.catalina.bottle.tar.gz%250A%25F0%259F%258D%25BA%2520%2520%252Fusr%252Flocal%252FCellar%252Fneofetch%252F7.1.0%253A%25206%2520files%252C%2520351.5KB
[Pacman Rosetta]: https://wiki.archlinux.org/index.php/Pacman/Rosetta
[icy/pacapt]: https://github.com/icy/pacapt
[pacapt/#117]: https://github.com/icy/pacapt/issues/117
[pacapt/#126]: https://github.com/icy/pacapt/issues/126
[rmtree]: https://github.com/beeftornado/homebrew-rmtree
[gsudo]: https://github.com/gerardog/gsudo
[rs-dev]: https://github.com/rami3l/pacaptr/tree/rs-dev
