extern crate quote;
extern crate syn;
extern crate proc_macro;
extern crate proc_macro2;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::Visibility;

#[derive(Debug)]
struct OnlyVis {
    pub vis: Visibility,
}

impl Parse for OnlyVis {
    fn parse(input: ParseStream) -> Result<Self> {
        let vis = input.parse::<Visibility>().expect("TAG_A");
        Ok(OnlyVis { vis })
    }
}

impl ToTokens for OnlyVis {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.vis.to_tokens(tokens);
    }
}

// noinspection DuplicatedCode
fn analyze_parse<T: Parse + std::fmt::Debug>(stream: proc_macro2::TokenStream) -> Result<T> {
    println!("New macro invocation");
    println!("====================");
    println!("string:  {:?}", stream.to_string());
    println!("stream2: {:?}", stream);
    println!("empty? {}", stream.is_empty());
    let res = syn::parse2::<T>(stream);
    println!("parsed: {:?}", res);
    println!();
    res
}

#[proc_macro]
pub fn mac_internal(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input2 = proc_macro2::TokenStream::from(input);

    let vis = analyze_parse::<Visibility>(input2).expect("TAG_B");

    let item = quote! {
        #vis const _: u32 = 0;
    };
    item.into()
}
