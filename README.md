# AnyLang - Static Localization for Rust

A Rust proc-macro crate for embedding localization files directly into your binary at compile time.
Supports JSON format with TOML support planned for future releases.

## Features

- **Zero-runtime overhead** - All translations are compiled into your binary
- **Type-safe** - Full Rust type checking for all localized strings
- **Hierarchical organization** - Nested JSON objects become nested Rust modules
- **Multi-format support** - JSON with TOML coming soon
- **Flexible data types** - Supports strings, numbers, booleans, arrays, and null values

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
anylang = { version = "0.1", features = ["json"] }
```

## Usage

### Basic JSON Localization

You can do the arrangement differently, but for convenience I will do it like this:

```
main.rs
lang/
├── en_US.json
├── ru_RU.json
└── de_DE.json
```

**en_US.json:**

```json
{
    "ping": "pong",
    "dummy": {
        "foo": "buzz",
        "some": ["none", "or", 0]
    },
    "rust": {
        "rust": "rust",
        "is": null,
        "good": {
            "true": [1, true]
        }
    }
}
```

**ru_RU.json:**

```json
{
    "ping": "понг",
    "dummy": {
        "foo": "базз",
        "some": ["ничего", "или", 0]
    },
    "rust": {
        "rust": "раст",
        "is": null,
        "good": {
            "true": [1, true]
        }
    }
}
```

**de_DE.json:**

```json
228.01
```

These examples work in different variations of the main.rs file:

```rust
use anylang::include_json_dir;

// Include English translations
include_json_dir!("./lang", "en_US");

fn main() {
    assert_eq!(lang::PING, "pong");
    assert_eq!(lang::dummy::FOO, "buzz");
    assert_eq!(lang::dummy::SOME, ["none", "or", "0"]);
    assert_eq!(lang::rust::RUST, "rust");
    assert!(lang::rust::IS.is_empty());
    assert_eq!(lang::rust::good::TRUE, ["1", "true"]);
}
```

```rust
use anylang::include_json_dir;

// Include Russian translations
include_json_dir!("./lang", "ru_RU");

fn main() {
    assert_eq!(lang::PING, "понг");
    assert_eq!(lang::dummy::FOO, "базз");
    assert_eq!(lang::dummy::SOME, ["ничего", "или", "0"]);
    assert_eq!(lang::rust::RUST, "раст");
    assert!(lang::rust::IS.is_empty());
    assert_eq!(lang::rust::good::TRUE, ["1", "true"]);
}
```

```rust
use anylang::include_json_dir;

// Include German translations (simple value)
include_json_dir!("./lang", "de_DE");

fn main() {
    assert_eq!(lang::DE_DE, "228.01");
}
```

## JSON Array Support

AnyLang also supports JSON arrays as root elements:

**en_US.json:**

```json
[
    {
        "ping": "pong"
    },
    {
        "dummy": {
            "some": ["none", "or", 0]
        },
        "foo": "buzz",
        "rust": {
            "rust": "rust",
            "is": null,
            "good": {
                "true": [1, true]
            }
        }
    }
]
```

## Type Conversion

All JSON types are automatically converted to Rust string types:

- **String** → `&'static str`
- **Number** → `&'static str` (string representation)
- **Boolean** → `&'static str` ("true" or "false")
- **Null** → `&'static str` (empty string)
- **Array** → `[&'static str; N]`
- **Object** → Rust module with constants

## Naming Convention

JSON keys are converted to SCREAMING_SNAKE_CASE for Rust constants:

- `"foo_bar"` becomes `FOO_BAR`
- `"some_key"` becomes `SOME_KEY`

## Roadmap

- [x] JSON support
- [ ] TOML support
- [ ] YAML support
