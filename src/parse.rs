use quote::quote;
use std::{fs, path};

#[derive(PartialEq)]
enum TokenVariant {
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

#[cfg(feature = "json")]
#[derive(PartialEq)]
enum TokenJson {
    Namespace(JsonNamespace),
    Token(Token),
}

#[cfg(feature = "json")]
impl From<Token> for TokenJson {
    fn from(value: Token) -> Self {
        Self::Token(value)
    }
}

#[cfg(feature = "json")]
impl From<JsonNamespace> for TokenJson {
    fn from(value: JsonNamespace) -> Self {
        Self::Namespace(value)
    }
}

#[cfg(feature = "json")]
impl From<TokenJson> for proc_macro2::TokenStream {
    fn from(val: TokenJson) -> Self {
        match val {
            TokenJson::Namespace(namespace) => namespace.into(),
            TokenJson::Token(token) => {
                let name =
                    syn::Ident::new(&token.name.to_uppercase(), proc_macro2::Span::call_site());
                let ty = token.data.get_type();
                let value = token.data.into_data();

                quote! { pub const #name: #ty = #value; }
            },
        }
    }
}

#[cfg(feature = "json")]
#[derive(Default, PartialEq)]
struct JsonNamespace {
    namespace: Option<String>,
    tokens: Vec<TokenJson>,
}

#[cfg(feature = "json")]
impl JsonNamespace {
    fn new<T>(namespace: T) -> Self
    where
        String: From<T>,
    {
        let namespace = String::from(namespace);

        Self {
            namespace: Some(namespace),
            ..Default::default()
        }
    }
}

#[cfg(feature = "json")]
impl From<JsonNamespace> for proc_macro2::TokenStream {
    fn from(val: JsonNamespace) -> Self {
        let mod_name = syn::Ident::new(
            &val.namespace.unwrap_or("lang".to_owned()),
            proc_macro2::Span::call_site(),
        );
        let mods_and_consts = val
            .tokens
            .into_iter()
            .map(Into::<proc_macro2::TokenStream>::into);

        quote! {
            pub mod #mod_name {
                #(#mods_and_consts)*
            }
        }
    }
}

pub(crate) struct File {
    #[allow(unused)]
    pub(crate) name: String,
    #[cfg(feature = "json")]
    tokens: JsonNamespace,
}

#[cfg(feature = "json")]
impl From<File> for proc_macro2::TokenStream {
    fn from(val: File) -> Self {
        val.tokens.into()
    }
}

#[cfg(feature = "json")]
fn parse_json(
    value: &serde_json::Value,
    root: &mut JsonNamespace,
    file_name: &str,
) -> syn::Result<()> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                match val {
                    serde_json::Value::Object(_) => {
                        let mut namespace = JsonNamespace::new(key);

                        parse_json(val, &mut namespace, file_name)?;

                        root.tokens.push(TokenJson::from(namespace))
                    },
                    serde_json::Value::Array(arr) => {
                        let mut tokens = Vec::with_capacity(arr.len());

                        for val in arr {
                            if matches!(val, serde_json::Value::Object(_)) {
                                return Err(syn::Error::new_spanned(
                                    syn::LitStr::new(
                                        &val.to_string(),
                                        proc_macro2::Span::call_site(),
                                    ),
                                    "Everything except Object was expected",
                                ));
                            }

                            if let serde_json::Value::String(s) = val {
                                tokens.push(s.to_owned());
                            } else {
                                tokens.push(val.to_string())
                            }
                        }

                        root.tokens.push(TokenJson::from(Token {
                            name: key.to_owned(),
                            data: TokenVariant::from_iter(tokens),
                        }))
                    },
                    serde_json::Value::String(s) => {
                        root.tokens.push(TokenJson::from(Token {
                            name: key.to_owned(),
                            data: TokenVariant::from_str(s),
                        }))
                    },
                    serde_json::Value::Number(n) => {
                        root.tokens.push(TokenJson::from(Token {
                            name: key.to_owned(),
                            data: TokenVariant::from_str(n),
                        }))
                    },
                    serde_json::Value::Null => {
                        root.tokens.push(TokenJson::from(Token {
                            name: key.to_owned(),
                            data: TokenVariant::from_str(String::new()),
                        }))
                    },
                    serde_json::Value::Bool(b) => {
                        root.tokens.push(TokenJson::from(Token {
                            name: key.to_owned(),
                            data: TokenVariant::from_str(b),
                        }))
                    },
                }
            }
        },
        serde_json::Value::Array(arr) => {
            for val in arr {
                if matches!(val, serde_json::Value::Object(_)) {
                    parse_json(val, root, file_name)?;
                } else {
                    return Err(syn::Error::new_spanned(
                        syn::LitStr::new(&val.to_string(), proc_macro2::Span::call_site()),
                        format!("Excepted Object into Array, but actually {val:?}"),
                    ));
                }
            }
        },
        serde_json::Value::String(s) => {
            root.tokens.push(TokenJson::from(Token {
                name: file_name.to_owned(),
                data: TokenVariant::from_str(s),
            }));
        },
        serde_json::Value::Number(n) => {
            root.tokens.push(TokenJson::from(Token {
                name: file_name.to_owned(),
                data: TokenVariant::from_str(n),
            }));
        },
        serde_json::Value::Null => {
            root.tokens.push(TokenJson::from(Token {
                name: file_name.to_owned(),
                data: TokenVariant::from_str(String::new()),
            }));
        },
        serde_json::Value::Bool(b) => {
            root.tokens.push(TokenJson::from(Token {
                name: file_name.to_owned(),
                data: TokenVariant::from_str(b),
            }));
        },
    }

    Ok(())
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
