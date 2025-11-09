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
