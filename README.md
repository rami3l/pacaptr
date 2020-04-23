# pacaptr

- [pacaptr](#pacaptr)
  - [Introduction](#introduction)
  - [Warning: WIP](#warning-wip)
  - [Building](#building)
  - [Implemented Features](#implemented-features)

## Introduction

`pacapt` is a wrapper for many package managers with pacman-style command syntax.

_Note: To start with, we choose to focus on `homebrew`. Support for more package managers will be added Soon™._

Use one syntax to rule them all!

## Warning: WIP

This is an experimental port of [icy/pacapt] in Golang.

~~We choose Golang for better readability, better testing, and hopefully without loss of portability.~~

*I finally choose to go on with Rust instead of Golang. You can check it out right now in the [rs-dev] branch.*

Now the implementations of different package managers are all placed in `dispatch` package, with names like `impl_xxx.go`.

To play along at home:

```bash
git clone https://github.com/rami3l/pacaptr.git
cd pacaptr
```

... and then try something like:

```bash
go run main.go -S curl
```

## Building

To install:

```bash
go install "github.com/rami3l/pacaptr"
```

To uninstall:

```bash
go clean -i "github.com/rami3l/pacaptr"
```

We currently provide `go install` only.
PPAs might be added when appropriate.

## Implemented Features

- `Homebrew` support: Experimental.
  
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

- `Dpkg/Apt` support: Experimental.

- `--dryrun`, `--dry-run`: Use this flag to just print out the command to be executed (sometimes with a --dry-run flag to activate the package manager's dryrun option).

  - `#>` means that the following command will not be run, while `>>` means that it will be.

  - Some query commands might still be run, but anything "big" should have been stopped from running, eg. installation. For instance:

    ```bash
    # Nothing will be deleted here:
    pacaptr -Sc --dryrun
    #> brew cleanup --dry-run
    .. (showing the files to be removed)

    # Without `--dryrun`, the forementioned files will be removed:
    pacaptr -Sc
    >> brew cleanup
    .. (cleaning up)
    ```

- `--yes`, `--noconfirm`, `--no-confirm`: Use this flag to trigger the corresponding flag of your package manager (if possible) in order to answer "yes" to every incoming question. Potentially dangerous if you don't know what you're doing!

[icy/pacapt]: https://github.com/icy/pacapt
[rmtree]: https://github.com/beeftornado/homebrew-rmtree
[gsudo]: https://github.com/gerardog/gsudo
[rs-dev]: https://github.com/rami3l/pacaptr/tree/rs-dev
