//! # AnyLang - Static Localization for Rust
//!
//! A Rust proc-macro crate for embedding localization files directly into your
//! binary at compile time. Supports JSON format with TOML support planned for
//! future releases.
//!
//! ## Features
//!
//! - **Zero-runtime overhead** - All translations are compiled into your binary
//! - **Type-safe** - Full Rust type checking for all localized strings
//! - **Hierarchical organization** - Nested JSON objects become nested Rust
//!   modules
//! - **Multi-format support** - JSON with TOML coming soon
//! - **Flexible data types** - Supports strings, numbers, booleans, arrays, and
//!   null values
//!
//! ## Installation
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! anylang = { version = "0.1", features = ["json"] }
//! ```
//!
//! ## Usage
//!
//! ### Basic JSON Localization
//!
//! You can do the arrangement differently, but for convenience I will do it
//! like this:
//!
//! ```
//! main.rs
//! lang/
//! ├── en_US.json
//! ├── ru_RU.json
//! └── de_DE.json
//! ```
//!
//! **en_US.json:**
//!
//! ```json
//! {
//!     "ping": "pong",
//!     "dummy": {
//!         "foo": "buzz",
//!         "some": ["none", "or", 0]
//!     },
//!     "rust": {
//!         "rust": "rust",
//!         "is": null,
//!         "good": {
//!             "true": [1, true]
//!         }
//!     }
//! }
//! ```
//!
//! **ru_RU.json:**
//!
//! ```json
//! {
//!     "ping": "понг",
//!     "dummy": {
//!         "foo": "базз",
//!         "some": ["ничего", "или", 0]
//!     },
//!     "rust": {
//!         "rust": "раст",
//!         "is": null,
//!         "good": {
//!             "true": [1, true]
//!         }
//!     }
//! }
//! ```
//!
//! **de_DE.json:**
//!
//! ```json
//! 228.01
//! ```
//!
//! These examples work in different variations of the main.rs file:
//!
//! ```rust
//! use anylang::include_json_dir;
//!
//! // Include English translations
//! include_json_dir!("./lang", "en_US");
//!
//! fn main() {
//!     assert_eq!(lang::PING, "pong");
//!     assert_eq!(lang::dummy::FOO, "buzz");
//!     assert_eq!(lang::dummy::SOME, ["none", "or", "0"]);
//!     assert_eq!(lang::rust::RUST, "rust");
//!     assert!(lang::rust::IS.is_empty());
//!     assert_eq!(lang::rust::good::TRUE, ["1", "true"]);
//! }
//! ```
//!
//! ```rust
//! use anylang::include_json_dir;
//!
//! // Include Russian translations
//! include_json_dir!("./lang", "ru_RU");
//!
//! fn main() {
//!     assert_eq!(lang::PING, "понг");
//!     assert_eq!(lang::dummy::FOO, "базз");
//!     assert_eq!(lang::dummy::SOME, ["ничего", "или", "0"]);
//!     assert_eq!(lang::rust::RUST, "раст");
//!     assert!(lang::rust::IS.is_empty());
//!     assert_eq!(lang::rust::good::TRUE, ["1", "true"]);
//! }
//! ```
//!
//! ```rust
//! use anylang::include_json_dir;
//!
//! // Include German translations (simple value)
//! include_json_dir!("./lang", "de_DE");
//!
//! fn main() {
//!     assert_eq!(lang::DE_DE, "228.01");
//! }
//! ```
//!
//! ## JSON Array Support
//!
//! AnyLang also supports JSON arrays as root elements:
//!
//! **en_US.json:**
//!
//! ```json
//! [
//!     {
//!         "ping": "pong"
//!     },
//!     {
//!         "dummy": {
//!             "some": ["none", "or", 0]
//!         },
//!         "foo": "buzz",
//!         "rust": {
//!             "rust": "rust",
//!             "is": null,
//!             "good": {
//!                 "true": [1, true]
//!             }
//!         }
//!     }
//! ]
//! ```
//!
//! ## Type Conversion
//!
//! All JSON types are automatically converted to Rust string types:
//!
//! - **String** → `&'static str`
//! - **Number** → `&'static str` (string representation)
//! - **Boolean** → `&'static str` ("true" or "false")
//! - **Null** → `&'static str` (empty string)
//! - **Array** → `[&'static str; N]`
//! - **Object** → Rust module with constants
//!
//! ## Naming Convention
//!
//! JSON keys are converted to SCREAMING_SNAKE_CASE for Rust constants:
//!
//! - `"foo_bar"` becomes `FOO_BAR`
//! - `"some_key"` becomes `SOME_KEY`
//!
//! ## Roadmap
//!
//! - [x] JSON support
//! - [ ] TOML support
//! - [ ] YAML support
//!
//! ## License
//!
//! **MIT**

mod parse;
#[macro_use]
mod r#macro;

use proc_macro::TokenStream;
use std::fs;
use syn::{LitStr, Token, parse::Parse, parse_macro_input};

pub(crate) const CRATE_NAME: &str = "anylang";

struct MacroArgs {
    dir_path: LitStr,
    lang: LitStr,
}

impl Parse for MacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let dir_path = input.parse::<LitStr>()?;

        let lang = if input.peek(Token![,]) {
            let _comma: Token![,] = input.parse()?;
            input.parse::<LitStr>()?
        } else {
            const ERR_MSG: &str = "Failed to get language";

            return Err(syn::Error::new_spanned(
                LitStr::new(ERR_MSG, proc_macro2::Span::call_site()),
                error!(ERR_MSG),
            ));
        };

        Ok(Self { dir_path, lang })
    }
}

impl MacroArgs {
    fn dir_path(&self) -> String {
        self.dir_path.value()
    }

    fn lang(&self) -> String {
        self.lang.value()
    }
}

/// **Example of usage:**
///
/// ```json
/// {
///     "ping": "понг",
///     "dummy": {
///         "foo": "базз",
///         "some": ["ничего", "или", 0]
///     },
///     "rust": {
///         "rust": "раст",
///         "is": null,
///         "good": {
///             "true": [1, true]
///         }
///     }
/// }
/// ```
///
/// ```rust
/// use anylang::include_json_dir;
///
/// // Include Russian translations
/// include_json_dir!("./lang", "ru_RU");
///
/// fn main() {
///     assert_eq!(lang::PING, "понг");
///     assert_eq!(lang::dummy::FOO, "базз");
///     assert_eq!(lang::dummy::SOME, ["ничего", "или", "0"]);
///     assert_eq!(lang::rust::RUST, "раст");
///     assert!(lang::rust::IS.is_empty());
///     assert_eq!(lang::rust::good::TRUE, ["1", "true"]);
/// }
/// ```
#[cfg(feature = "json")]
#[proc_macro]
pub fn include_json_dir(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as MacroArgs);
    let dir_path = args.dir_path();
    let lang = args.lang();

    let dir_entries = match fs::read_dir(&dir_path) {
        Ok(entries) => entries,
        Err(e) => {
            return syn::Error::new_spanned(
                LitStr::new(&dir_path, proc_macro2::Span::call_site()),
                error!(format!("Failed to read directory: {e}")),
            )
            .to_compile_error()
            .into();
        },
    };

    let mut needed_file = None;

    for entry in dir_entries.flatten() {
        let path = entry.path();

        if let Some(file_name) = path.file_prefix()
            && file_name == lang.as_str()
        {
            match parse::parse_from_file(path) {
                Ok(file) => needed_file = Some(file),
                Err(e) => return e.to_compile_error().into(),
            }
            break;
        }
    }

    if let Some(file) = needed_file {
        Into::<proc_macro2::TokenStream>::into(file).into()
    } else {
        syn::Error::new_spanned(
            LitStr::new(&lang, proc_macro2::Span::call_site()),
            error!(format!(
                "Failed to get file with name {lang} in directory {dir_path}"
            )),
        )
        .into_compile_error()
        .into()
    }
}
