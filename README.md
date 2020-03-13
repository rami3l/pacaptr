# pacapt-go

## Warning: WIP

This is an experimental port of [icy/pacapt] in Golang. We chose Golang for better readability, better testing, and hopefully without loss of portability.

Now the implementations of different package managers are all placed in `dispatch` package, and we're implementing `homebrew` to start with.

## Introduction

`pacapt` is a wrapper for many package managers.

Simply install packages (eg. `htop`) with `pacapt -S htop` or `pacapt install htop` on any `Linux`, `BSD`, or `macOS` machine.

Use one syntax to rule them all!

[icy/pacapt]: https://github.com/icy/pacapt
