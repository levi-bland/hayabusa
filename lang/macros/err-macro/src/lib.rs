// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0\

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, LitStr, Token,
};

struct ErrInput {
    fmt: LitStr,
    code: Expr,
    args: Vec<Expr>,
}

impl Parse for ErrInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fmt: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let code: Expr = input.parse()?;

        let mut args = Vec::new();
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            args.push(input.parse::<Expr>()?);
        }

        Ok(ErrInput { fmt, code, args })
    }
}

struct LogInput {
    fmt: LitStr,
    args: Vec<Expr>,
}

impl Parse for LogInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fmt: LitStr = input.parse()?;
        match input.parse::<Token![,]>() {
            Ok(_) => (),
            Err(_) => {
                return Ok(LogInput {
                    fmt,
                    args: Vec::new(),
                })
            }
        }

        let mut args = Vec::new();
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            args.push(input.parse::<Expr>()?);
        }

        Ok(LogInput { fmt, args })
    }
}

/// Splits "foo {} bar {} baz" into ["foo ", " bar ", " baz"]
/// Returns Err if placeholder count != args count.
fn split_fmt(fmt: &str, arg_count: usize, span: Span) -> syn::Result<Vec<String>> {
    let segments: Vec<&str> = fmt.split("{}").collect();
    let placeholder_count = segments.len() - 1;

    if placeholder_count != arg_count {
        return Err(syn::Error::new(
            span,
            format!(
                "format string has {placeholder_count} placeholder(s) but {arg_count} argument(s) were supplied"
            ),
        ));
    }

    Ok(segments.iter().map(|s| s.to_string()).collect())
}

#[proc_macro]
pub fn err(input: TokenStream) -> TokenStream {
    let ErrInput { fmt, code, args } = parse_macro_input!(input as ErrInput);

    let fmt_str = fmt.value();
    let span = fmt.span();

    let segments = match split_fmt(&fmt_str, args.len(), span) {
        Ok(s) => s,
        Err(e) => return e.to_compile_error().into(),
    };

    // Build interleaved append calls:
    // append(seg[0]), append(arg[0]), append(seg[1]), append(arg[1]), ..., append(seg[n])
    let mut appends = Vec::new();

    for (i, segment) in segments.iter().enumerate() {
        if !segment.is_empty() {
            appends.push(quote! { logger.append(#segment); });
        }
        if i < args.len() {
            let arg = &args[i];
            appends.push(quote! { logger.append(#arg); });
        }
    }

    quote! {{
        #[cfg(feature = "debug-logs")]
        {
            let mut logger = Logger::<200>::default();
            #(#appends)*
            logger.append(" [");
            logger.append(file!());
            logger.append(":");
            logger.append(line!());
            logger.append("]");
            logger.log();
        }

        return Err(ProgramError::from(#code));
    }}
    .into()
}

#[proc_macro]
pub fn log(input: TokenStream) -> TokenStream {
    let LogInput { fmt, args } = parse_macro_input!(input as LogInput);

    let fmt_str = fmt.value();
    let span = fmt.span();

    let segments = match split_fmt(&fmt_str, args.len(), span) {
        Ok(s) => s,
        Err(e) => return e.to_compile_error().into(),
    };

    // Build interleaved append calls:
    // append(seg[0]), append(arg[0]), append(seg[1]), append(arg[1]), ..., append(seg[n])
    let mut appends = Vec::new();

    for (i, segment) in segments.iter().enumerate() {
        if !segment.is_empty() {
            appends.push(quote! { logger.append(#segment); });
        }
        if i < args.len() {
            let arg = &args[i];
            appends.push(quote! { logger.append(#arg); });
        }
    }

    quote! {
        {
            let mut logger = ::hayabusa::prelude::pinocchio_log::logger::Logger::<200>::default();
            #(#appends)*
            logger.log();
        }
    }
    .into()
}
