# pacapt-go

## Introduction

`pacapt` is a wrapper for many package managers with pacman-style command syntax.

_Note: To start with, we choose to focus on `homebrew`. Support for more package managers will be added Soon™._

Use one syntax to rule them all!

## Warning: WIP

This is an experimental port of [icy/pacapt] in Golang. We choose Golang for better readability, better testing, and hopefully without loss of portability.

Now the implementations of different package managers are all placed in `dispatch` package, with names like `impl_xxx.go`.

To play along at home:

```bash
git clone https://github.com/rami3l/pacapt-go.git
cd pacapt-go
```

... and then try something like:

```bash
go run main.go -S curl
```

## Installation

```bash
# To install
go install "github.com/rami3l/pacapt-go"

# To uninstall
go clean -i "github.com/rami3l/pacapt-go"
```

We currently provide `go install` only.
PPAs might be added when appropriate.

## Implemented Features

- `Homebrew` support: Experimental, with automatic `brew cask` invocation implemented for `-S`, `-R`, `-Su`, and more.
  
    ```bash
    go run main.go -S curl --dryrun
    >> brew install curl

    go run main.go -S gimp --dryrun
    >> brew cask install gimp
    ```

- `Chocolatey` support: Experimental. Don't forget to run in an elevated shell!

- `Dpkg/Apt` support: Experimental.
- `--dryrun`: An option that will try to just print out the command to be executed. Some query commands might still be run, but anything "big" should have been stopped from running, eg. installation.

[icy/pacapt]: https://github.com/icy/pacapt
