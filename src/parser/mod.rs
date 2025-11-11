#[cfg(feature = "json")]
mod json;

use std::{fs, path};

#[cfg(feature = "json")]
use json::*;

#[derive(PartialEq)]
pub enum TokenVariant {
    Single(String),
    Array(Vec<String>),
}

impl TokenVariant {
    fn into_data(self) -> proc_macro2::TokenStream {
        match self {
            Self::Single(s) => {
                let lit = syn::LitStr::new(&s, proc_macro2::Span::call_site());
                quote::quote!(#lit)
            },
            Self::Array(arr) => {
                let items = arr
                    .iter()
                    .map(|s| syn::LitStr::new(s, proc_macro2::Span::call_site()));
                quote::quote!([#(#items),*])
            },
        }
    }

    fn get_type(&self) -> syn::Type {
        match self {
            Self::Single(_) => {
                syn::Type::Reference(syn::TypeReference {
                    and_token: syn::parse_str("&").unwrap(),
                    lifetime: None,
                    mutability: None,
                    elem: syn::parse_str("str").unwrap(),
                })
            },
            Self::Array(arr) => syn::parse_str(&format!("[&str; {}]", arr.len())).unwrap(),
        }
    }
}

macro_rules! error {
    ($err:expr) => {
        format!("[{}:parse:ERROR] {}", crate::CRATE_NAME, $err)
    };
}

impl TokenVariant {
    fn from_str<T>(value: T) -> Self
    where
        T: ToString,
    {
        Self::Single(value.to_string())
    }

    fn from_iter<I>(value: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        Self::Array(value.into_iter().collect())
    }
}

#[derive(PartialEq)]
struct Token {
    name: String,
    data: TokenVariant,
}

pub(crate) struct File {
    #[allow(unused)]
    pub(crate) name: String,
    #[cfg(feature = "json")]
    tokens: JsonNamespace,
}

pub(crate) fn parse_from_file(file: path::PathBuf) -> syn::Result<File> {
    let file_name = if let Some(file_name) = file.file_prefix() {
        file_name.to_string_lossy().to_string()
    } else {
        return Err(syn::Error::new_spanned(
            syn::LitStr::new(
                file.to_string_lossy().as_ref(),
                proc_macro2::Span::call_site(),
            ),
            error!(format!(
                "File expected but received {}",
                file.to_string_lossy().to_string()
            )),
        ));
    };

    if let Some(extension) = file.extension() {
        #[cfg(feature = "json")]
        if extension == "json" {
            let data = fs::File::open(file).map_err(|e| {
                syn::Error::new_spanned(
                    syn::LitStr::new(&e.to_string(), proc_macro2::Span::call_site()),
                    error!(format!("Cannot read file {file_name} cause {e}")),
                )
            })?;
            let value = serde_json::from_reader(data).map_err(|e| {
                syn::Error::new_spanned(
                    syn::LitStr::new(&e.to_string(), proc_macro2::Span::call_site()),
                    error!(format!("Cannot deserialize {file_name} cause {e}")),
                )
            })?;

            let mut root_namespace = JsonNamespace::default();
            parse_json(&value, &mut root_namespace, &file_name.to_uppercase())?;

            return Ok(File {
                name: file_name,
                tokens: root_namespace,
            });
        } else {
            return Ok(File {
                name: file_name,
                tokens: JsonNamespace::default(),
            });
        }

        const ERR_MSG: &str = "No one feature was choosed!";

        #[allow(unused)]
        Err(syn::Error::new_spanned(
            syn::LitStr::new(ERR_MSG, proc_macro2::Span::call_site()),
            error!(ERR_MSG),
        ))
    } else {
        Err(syn::Error::new_spanned(
            syn::LitStr::new(&file_name, proc_macro2::Span::call_site()),
            error!("A file with some extension was expected"),
        ))
    }
}
