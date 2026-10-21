# Lunify

[![CI](https://github.com/ve5li/lunify/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ve5li/lunify/actions/workflows/ci.yml)
[![Test Coverage](https://raw.githubusercontent.com/gist/vE5li/173a7aa9ac7d5e82e238048e06aa99c2/raw/lunify-coverage-badge.svg)](https://github.com/ve5li/lunify/actions/workflows/coverage.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/lunify.svg)](https://crates.io/crates/lunify)

A crate for converting Lua byte code to different versions and formats.

Currently Lua 5.0 and Lua 5.1 are supported inputs.

# Example

```rust
use lunify::{Format, LunifyError, Endianness, BitWidth, unify};

// Lua byte code in any suppored format
let input_bytes = include_bytes!("../test_files/lua50.luab");

// Desired output format. May specify pointer width, endianness, sizes of datatypes, ...
let output_format = Format {
    endianness: Endianness::Little,
    // Convert from byte code that runs on a 32 bit machine to byte code that runs on a 64 bit machine
    size_t_width: BitWidth::Bit64,
    ..Format::default()
};

// Convert input bytes to the desired format
let output_bytes = unify(input_bytes, &output_format, &Default::default());
```

---

<p align="center"><sub>꧁∙·▫ₒₒ▫ᵒᴼᵒ▫ₒₒ▫꧁ project maintained by <a href="https://github.com/vE5li/repository-manager"><code>repository-manager</code></a> ꧂▫ₒₒ▫ᵒᴼᵒ▫ₒₒ▫·∙꧂</sub></p>
