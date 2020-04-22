#![allow(unused, dead_code)]
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::{Ident, Visibility};
use syn::parse::{Parse, ParseStream, Result};

mod with_ident;
mod only_vis;

// play with it:
//  - reorder the rules,
//  - remove the one without visibility modifier,
//  - remove all token trees except visibility modifier



#[derive(Debug)]
struct VisIdent {
    pub vis: Visibility,
    pub ident: Ident,
}

impl Parse for VisIdent {
    fn parse(input: ParseStream) -> Result<Self> {
        let vis = input.parse::<Visibility>().expect("TAG_1");
        let ident = input.parse::<Ident>().expect("TAG_2");
        Ok(VisIdent { vis, ident })
    }
}

impl ToTokens for VisIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.vis.to_tokens(tokens);
        self.ident.to_tokens(tokens);
    }
}



// noinspection DuplicatedCode
fn analyze_parse<T>(header: &str, stream: proc_macro2::TokenStream) -> Result<T>
    where T: syn::parse::Parse + std::fmt::Debug
{
    println!("New parse invocation");
    println!("====================");
    println!("header: {}", header);
    println!("string:  {:?}", stream.to_string());
    println!("stream2: {:?}", stream);
    println!("empty? {}", stream.is_empty());
    let res = syn::parse2::<T>(stream);
    println!("parsed: {:?}", res);
    println!();
    res
}


// construct parts of TokenStream

fn append_blank_vis(stream: &mut TokenStream) {
    let empty_group = Group::new(Delimiter::None, TokenStream::new());
    stream.append(empty_group);
}

fn append_pub_crate_vis(stream: &mut TokenStream) {
    // make sure it is only Visibility and nothing else
    let vis: Visibility = syn::parse_quote!( pub(crate) );
    vis.to_tokens(stream);
}

fn append_some_ident(stream: &mut TokenStream) {
    stream.append(Ident::new("foobar", proc_macro2::Span::call_site()));
}


// reconstruction of the whole TokenStream

fn reconstruct_empty_token_stream() -> TokenStream {
    TokenStream::new()
}

fn reconstruct_blank_vis_token_stream() -> TokenStream {
    let mut stream = reconstruct_empty_token_stream();
    append_blank_vis(&mut stream);
    stream
}

fn reconstruct_ident_only_token_stream() -> TokenStream {
    let mut stream = reconstruct_empty_token_stream();
    append_some_ident(&mut stream);
    stream
}

fn reconstruct_vis_ident_token_stream() -> TokenStream {
    let mut stream = reconstruct_empty_token_stream();
    append_pub_crate_vis(&mut stream);
    append_some_ident(&mut stream);
    stream
}

fn reconstruct_blank_vis_with_ident_token_stream() -> TokenStream {
    let mut stream = reconstruct_blank_vis_token_stream();
    append_some_ident(&mut stream);
    stream
}


fn main() {
    let stream = reconstruct_empty_token_stream();
    analyze_parse::<Visibility>("empty stream", stream).ok();
    // Ok(Inherited)

    let stream = reconstruct_blank_vis_token_stream();
    analyze_parse::<Visibility>("blank vis (empty group)", stream).ok();
    // Err(Error("unexpected token"))
    // WTF is blank group

    let stream = reconstruct_ident_only_token_stream();
    analyze_parse::<VisIdent>("no vis, only ident", stream).ok();
    // Ok(VisIdent { vis: Inherited, ident: Ident(foobar) })

    let stream = reconstruct_vis_ident_token_stream();
    analyze_parse::<VisIdent>("pub vis, with ident", stream).ok();
    // Ok(VisIdent { vis: Restricted(..), ident: Ident(foobar) })

    let stream = reconstruct_blank_vis_with_ident_token_stream();
    analyze_parse::<VisIdent>("blank vis (empty group), with ident", stream).ok();
    // Ok(VisIdent { vis: Inherited, ident: Ident(foobar) })
    // WTF?! blank group was an error just few lines above
}
