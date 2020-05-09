# pacaptr

- [pacaptr](#pacaptr)
  - [Introduction](#introduction)
  - [Warning: WIP](#warning-wip)
  - [Running & Building](#running--building)
  - [Tips](#tips)

## Introduction

`pacaptr` is a Rust port of [icy/pacapt], a wrapper for many package managers with pacman-style command syntax.

`pacaptr` currently supports the following package managers:

- macOS/homebrew
- Windows/chocolatey
- Debian/dpkg
- Alpine/apk

Support for more package managers will be added Soon™.

## Warning: WIP

I choose Rust for better readability, better testing, and hopefully without loss of portability.

Now the implementations of different package managers are all placed in `./src/packmanager` folder, with names like `homebrew.rs`.

## Running & Building

We currently provide `cargo install` only.
PPAs might be added when appropriate.

To play along at home:

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

## Tips

- `Homebrew` support: Please note that this is for macOS only, `Linuxbrew` is currently not supported.
  
  - Automatic `brew cask` invocation: implemented for `-S`, `-R`, `-Su`, and more.
  
    ```bash
    pacaptr -S curl --dryrun
    #> brew install curl

    pacaptr -S gimp --dryrun
    #> brew cask install gimp
    ```

  - The use of `brew cask` commands can also be enforced by adding a `--cask` flag. Useful when a bottle and a cask share the same name, eg. `docker`.
  
  - To use `-Rs`, you need to install [rmtree] first:

    ```bash
    brew tap beeftornado/rmtree
    ```

- `Chocolatey` support: Experimental.

  - Tips: Don't forget to run in an elevated shell! You can do this easily with tools like [gsudo].

- `--dryrun`, `--dry-run`: Use this flag to just print out the command to be executed (sometimes with a --dry-run flag to activate the package manager's dryrun option).

  - `#>` means that the following command will not be run, while `>>` means that it is being run.

  - Some query commands might still be run, but anything "big" should have been stopped from running, eg. installation. For instance:

    ```bash
    # Nothing will be installed,
    # as `brew install curl` won't be run:
    pacaptr -S curl --dryrun
    #> brew install curl

    # Nothing will be deleted here,
    # but `brew cleanup --dry-run` is actually running:
    pacaptr -Sc --dryrun
    >> brew cleanup --dry-run
    .. (showing the files to be removed)

    # To remove the forementioned files,
    # run the command above again without `--dryrun`:
    pacaptr -Sc
    >> brew cleanup
    .. (cleaning up)
    ```

- `--yes`, `--noconfirm`, `--no-confirm`: Use this flag to trigger the corresponding flag of your package manager (if possible) in order to answer "yes" to every incoming question.
  - This option is useful when you don't want to be asked during installation, for example.
  - ... But it can be potentially dangerous if you don't know what you're doing!

[icy/pacapt]: https://github.com/icy/pacapt
[rmtree]: https://github.com/beeftornado/homebrew-rmtree
[gsudo]: https://github.com/gerardog/gsudo
[rs-dev]: https://github.com/rami3l/pacaptr/tree/rs-dev
