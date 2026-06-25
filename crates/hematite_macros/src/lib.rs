use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

use crate::{
    deserialize::impl_deserialize_macro, packet::impl_packet, serialize::impl_serialize_macro,
};

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

#[proc_macro_derive(Packet, attributes(packet_name))]
pub fn deserialize_packet(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_packet(&ast)
}
