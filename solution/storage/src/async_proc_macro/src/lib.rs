extern crate proc_macro;
use proc_macro::TokenStream;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro2::Ident;
use syn::{punctuated::Punctuated, token::Comma, FnArg};

#[proc_macro_attribute]
pub fn make_non_blocking(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(item as syn::ItemFn);

    let blocking_fn_name = format_ident!("{}_blocking", input.sig.ident.to_string());
    let non_blocking_ident = input.sig.ident;

    // reassign blocking version to new name
    input.sig.ident = blocking_fn_name.clone();

    // assumes and removes first argument since that's the PgConn we'll pass
    let original_args = &input
        .sig
        .inputs
        .clone() // sadly i need this
        .into_iter()
        .skip(1)
        .collect::<Punctuated<FnArg, Comma>>();
    let original_return = &input.sig.output;
    let passed_args = original_args
        .clone()
        .into_iter()
        .map(|fn_arg| match fn_arg {
            FnArg::Typed(arg) => match arg.pat.as_ref() {
                syn::Pat::Ident(ident) => ident.ident.clone(),
                _ => panic!("only works with Ident types"),
            },
            _ => panic!("doesnt work with Receiver types"),
        })
        .collect::<Punctuated<Ident, Comma>>();

    let original_fn = &input;
    let output: TokenStream = quote! {
        #original_fn

        pub async fn #non_blocking_ident(pool: crate::pool::PgPool, #original_args) #original_return {
           tokio::task::spawn_blocking(move || {
                let conn = pool
                    .get()
                    .map_err(|_| crate::errors::Error::ConnectionPoolError("could not get db connection".into()))?;
                #blocking_fn_name(&conn, #passed_args)
           }).await?
        }
    }
    .into();

    output
}
