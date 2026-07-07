use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

use crate::{
    all_packets::impl_all_packets, deserialize::impl_deserialize_macro,
    serialize::impl_serialize_macro,
};

mod all_packets;
mod deserialize;
mod packet;
mod serialize;

#[proc_macro_derive(Serialize)]
pub fn serialize_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_serialize_macro(&ast)
}

#[proc_macro_derive(Deserialize)]
pub fn deserialize_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_deserialize_macro(&ast)
}

#[proc_macro]
pub fn all_packets(input: TokenStream) -> TokenStream {
    impl_all_packets(input)
}
