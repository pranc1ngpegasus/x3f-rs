# x3f

## Overview

`x3f` is a Rust crate for parsing SIGMA Foveon X3F RAW files.

It focuses on safe and efficient container parsing and intentionally does not perform any image processing or color conversion.

<!-- From nearby, with respect for SIGMA -->

> [!NOTE]
> This crate is currently in an early, pre-alpha stage and the API is expected to evolve.

## Features

- zero runtime dependencies
- `no_std` compatible
  - https://docs.rust-embedded.org/book/intro/no-std.html
- Sans I/O pattern
  - parsing is fully decoupled from file or network I/O
  - https://sans-io.readthedocs.io
- zero-copy where possible
- panic-free by construction (returns errors instead of panicking)

## X3F format references

- libopenraw X3F format documentation  
  - https://libopenraw.freedesktop.org/formats/x3f/x3f-raw-format.pdf
