use quote::quote;

use super::*;

#[cfg(feature = "json")]
#[derive(PartialEq)]
pub(super) enum TokenJson {
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
pub(super) struct JsonNamespace {
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

#[cfg(feature = "json")]
impl From<File> for proc_macro2::TokenStream {
    fn from(val: File) -> Self {
        val.tokens.into()
    }
}

#[cfg(feature = "json")]
pub(super) fn parse_json(
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
