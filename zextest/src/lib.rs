extern crate proc_macro;

use std::fmt::Display;
use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro2::{Punct, Spacing, Span};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Expr, Ident, Lit, LitInt, LitStr, Token};

use zexrunner::*;

mod kw {
    syn::custom_keyword!(db);
    syn::custom_keyword!(tmsg);
    syn::custom_keyword!(tstr);
}

#[allow(dead_code)]
struct ZexTestCase {
    name: Ident,
    flag_mask: u8,
    base_state: ZexState,
    increment: ZexState,
    shift: ZexState,
    crc: u32,
    msg: LitStr,
}

// Skip comments until a condition is met
fn skip_comment<F: Fn(ParseStream) -> bool>(input: ParseStream, fun: F) -> Result<()> {
    if !input.peek(Token![;]) {
        return Ok(());
    }
    input.parse::<Token![;]>()?;
    while !input.is_empty() && !fun(input) {
        input.parse::<proc_macro2::TokenTree>()?;
    }
    Ok(())
}

// Try evaluating an expression that's either a literal number, 'msbt', 'msbtlo', 'msbthi', or a
// binary operation on those.
fn eval<N>(expr: &Expr) -> Result<N>
where
    N: std::ops::Neg<Output = N>,
    N: std::ops::Sub<Output = N>,
    N: From<u16>,
    N: FromStr,
    N::Err: Display,
{
    match expr {
        Expr::Unary(syn::ExprUnary {
            op: syn::UnOp::Neg(_),
            expr: e,
            ..
        }) => Ok(-(eval::<N>(e.as_ref())?)),
        Expr::Path(syn::ExprPath {
            path: syn::Path {
                leading_colon: None,
                segments,
            },
            ..
        }) => {
            if segments.len() != 1 {
                return Err(Error::new(expr.span(), format!("unknown symbol '{:?}'", expr)));
            }
            match &segments[0] {
                syn::PathSegment {
                    arguments: syn::PathArguments::None,
                    ident,
                } => match ident.to_string().as_ref() {
                    "msbt" => Ok(0x103u16.into()),
                    "msbthi" => Ok(0x1u16.into()),
                    "msbtlo" => Ok(0x3u16.into()),
                    symbol => Err(Error::new(expr.span(), format!("unknown symbol '{}'", symbol))),
                },
                _ => Err(Error::new(expr.span(), "unknown symbol")),
            }
        }
        Expr::Binary(syn::ExprBinary { left, right, op, .. }) => {
            let lv = eval::<N>(left)?;
            let rv = eval::<N>(right)?;
            match op {
                syn::BinOp::Sub(_) => Ok(lv - rv),
                _ => Err(Error::new(expr.span(), format!("unknown operator '{:?}'", op))),
            }
        }
        Expr::Lit(lit) => match &lit.lit {
            Lit::Int(int) => int.base10_parse::<N>(),
            _ => {
                println!("literal is {:?}", lit);
                Err(Error::new(expr.span(), "cannot parse literal"))
            }
        },
        _ => Err(Error::new(expr.span(), format!("cannot parse expression {:?}", expr))),
    }
}

fn parse_state(input: ParseStream) -> Result<ZexState> {
    let state = Punctuated::<Expr, Token![,]>::parse_separated_nonempty(input)?;
    if state.len() != 13 {
        return Err(Error::new(state.span(), "tstr requires exactly 13 elements"));
    }
    Ok(ZexState {
        instruction: ((eval::<i32>(&state[0])? as u32) << 24)
            | ((eval::<i32>(&state[1])? as u32) << 16)
            | ((eval::<i32>(&state[2])? as u32) << 8)
            | eval::<i32>(&state[3])? as u32,
        operand: eval::<i32>(&state[4])? as u16,
        iy: eval::<i32>(&state[5])? as u16,
        ix: eval::<i32>(&state[6])? as u16,
        hl: eval::<i32>(&state[7])? as u16,
        de: eval::<i32>(&state[8])? as u16,
        bc: eval::<i32>(&state[9])? as u16,
        f: eval::<i32>(&state[10])? as u8,
        a: eval::<i32>(&state[11])? as u8,
        sp: eval::<i32>(&state[12])? as u16,
    })
}

impl Parse for ZexTestCase {
    // Comment name: ZexState ZexState Comment ZexState Comment CRC Comment Message
    fn parse(input: ParseStream) -> Result<Self> {
        skip_comment(input, |stream| stream.peek2(Token![:]))?;

        // test name
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        // Flag mask
        input.parse::<kw::db>()?;
        let flag_mask = input.parse::<LitInt>()?.base10_parse::<u8>()?;

        // Base, increments, shifts
        skip_comment(input, |stream| stream.peek(kw::tstr))?;
        input.parse::<kw::tstr>()?;
        let base_state = parse_state(input)?;
        skip_comment(input, |stream| stream.peek(kw::tstr))?;
        input.parse::<kw::tstr>()?;
        let increment = parse_state(input)?;
        skip_comment(input, |stream| stream.peek(kw::tstr))?;
        input.parse::<kw::tstr>()?;
        let shift = parse_state(input)?;

        // expected crc
        skip_comment(input, |stream| stream.peek(kw::db))?;
        input.parse::<kw::db>()?;
        let crc1 = input.parse::<LitInt>()?.base10_parse::<u8>()?;
        input.parse::<Token![,]>()?;
        let crc2 = input.parse::<LitInt>()?.base10_parse::<u8>()?;
        input.parse::<Token![,]>()?;
        let crc3 = input.parse::<LitInt>()?.base10_parse::<u8>()?;
        input.parse::<Token![,]>()?;
        let crc4 = input.parse::<LitInt>()?.base10_parse::<u8>()?;
        let crc = ((crc1 as u32) << 24)
                | ((crc2 as u32) << 16)
                | ((crc3 as u32) << 8)
                | (crc4 as u32);

        // Test description
        skip_comment(input, |stream| stream.peek(kw::tmsg))?;
        input.parse::<kw::tmsg>()?;
        let msg = input.parse()?;

        Ok(ZexTestCase {
            name,
            flag_mask,
            base_state,
            increment,
            shift,
            crc: crc,
            msg,
        })
    }
}

#[allow(dead_code)]
fn let_bind<U>(tokens: &mut proc_macro2::TokenStream, name: &str, value: U)
where
    U: Into<proc_macro2::TokenTree>,
{
    tokens.append(Ident::new("let", Span::call_site()));
    tokens.append(Ident::new(name, Span::call_site()));
    tokens.append(Punct::new('=', Spacing::Alone));
    tokens.append(value);
    tokens.append(Punct::new(';', Spacing::Alone));
}

impl ToTokens for ZexTestCase {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let msg = &self.msg;
        let base = &self.base_state;
        let increment = &self.increment;
        let shift = &self.shift;
        let crc = &self.crc;
        tokens.append_all(quote! {
            let msg = #msg;
            let state = #base;
            let increment = #increment;
            let shift = #shift;
            let crc = #crc;
        });
    }
}

#[proc_macro]
pub fn testcase(input: TokenStream) -> TokenStream {
    let case = parse_macro_input!(input as ZexTestCase);
    let test_name = format_ident!("zex_{}", case.name);
    let flagmask = &case.flag_mask;
    let result = quote! {
        //#[test]
        fn #test_name() {
            #case
            assert_eq!(zex_run_test(&state, &increment, &shift, #flagmask), crc, "{}", msg);
        }
    };
    result.into()
}
