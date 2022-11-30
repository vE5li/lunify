# Lunify

[![Tests](https://github.com/ve5li/lunify/workflows/Tests/badge.svg)](https://github.com/ve5li/lunify/actions?query=workflow%3ATests)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/lunify.svg)](https://crates.io/crates/lunify)

A crate for converting Lua bytecode to different versions and formats.

Currently only Lua 5.0 and Lua 5.1 are supported inputs and Lua 5.0 support is limited.

# Example

```rust
use lunify::{Format, LunifyError, Endianness, BitWidth, unify};

// Lua bytecode in any suppored format
let input_bytes = include_bytes!("../test_files/lua50.luab");

// Desired output format. May specify pointer width, endianness, sizes of datatypes, ...
let output_format = Format {
    endianness: Endianness::Little,
    // Convert from bytecode that runs on a 32 bit machine to bytcode that runs on a 64 bit machine
    size_t_width: BitWidth::Bit64,
    ..Format::default()
};

// Convert input bytes to the desired format
let output_bytes = unify(input_bytes, &output_format, &Default::default());
```
