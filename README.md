# BestBefore

[![Crates.io](https://img.shields.io/crates/v/bestbefore.svg)](https://crates.io/crates/bestbefore)
[![Documentation](https://docs.rs/bestbefore/badge.svg)](https://docs.rs/bestbefore)
[![Build Status](https://github.com/suprematic/bestbefore/actions/workflows/rust.yml/badge.svg)](https://github.com/suprematic/bestbefore/actions/workflows/rust.yml)
[![Minimum Rust Version](https://img.shields.io/badge/MSRV-1.60.0-brightgreen.svg)](https://github.com/suprematic/bestbefore)
[![dependency status](https://deps.rs/repo/github/suprematic/bestbefore/status.svg)](https://deps.rs/repo/github/suprematic/bestbefore)

A Rust procedural macro for managing code with expiration dates.

## Introduction

Technical debt and code deprecation are persistent challenges in software development. As systems evolve, certain implementations become obsolete, APIs change, and better patterns emerge. However, identifying and removing deprecated code is often neglected, leading to maintenance burdens and unexpected bugs.

The `bestbefore` macro addresses this challenge by providing a compile-time mechanism to:

1. **Mark code with expiration dates** - Explicitly document when code should be reviewed or removed
2. **Enforce deadlines with compiler feedback** - Generate warnings or errors when code remains past its expiration date
3. **Manage technical debt systematically** - Create a clear lifecycle for deprecated code

Unlike comments or documentation that can be easily overlooked, this macro integrates with the Rust compiler to actively enforce code maintenance policies. It's especially valuable in larger teams and codebases where knowledge about legacy code may not be shared by all contributors.

## Overview

The `bestbefore` macro helps manage code deprecation by adding compile-time warnings or errors based on specified dates:

- When the compilation date is after the specified date, it generates a compiler warning
- When an optional `expires` parameter is provided and the compilation date is after that date, it causes a compilation failure

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
bestbefore = "0.1.0"
```

## Usage

```rust
use bestbefore::bestbefore;

// Generate a warning if compiled after March 2024
#[bestbefore("03.2024")]
fn legacy_function() {
    // ...
}

// Generate a warning if compiled after January 2023
// Cause a compilation error if compiled after December 2023
#[bestbefore("01.2023", expires = "12.2023")]
fn very_old_function() {
    // ...
}

// Add a custom message to the warning
#[bestbefore("02.2023", message = "Please use new_api() instead")]
fn deprecated_with_message() {
    // ...
}

// Apply to any kind of code block, not just functions
#[bestbefore("06.2023")]
mod legacy_module {
    // The entire module will generate a warning
}

// Apply to structs as well
#[bestbefore("05.2023")]
struct OldStructure {
    // ...
}
```

## Features

- **Date Format**: Uses the "MM.YYYY" format for simplicity
- **Multiple Target Types**: Can be applied to functions, modules, structs, traits, impls, and more
- **Custom Messages**: Supports custom warning and error messages
- **Environment Variable Override**: Set the `BESTBEFORE_DATE` environment variable to override the current date (useful for testing)
- **Date Validation**: The macro validates that expiration dates are always after warning dates

## Date Handling

The macro enforces several rules for date handling:

1. Warning dates are required and represent when code should be reviewed for potential replacement or removal
2. Expiration dates are optional and represent a hard deadline when the code must be removed
3. If an expiration date is provided, it must be after the warning date
4. The date format is "MM.YYYY" with the day assumed to be the first of the month

## License

Licensed under the Eclipse Public License 2.0 (EPL-2.0). 
