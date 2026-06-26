use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use serde::Deserialize;
use syn::{DeriveInput, Expr, Lit};

#[derive(Deserialize)]
struct Group {
    clientbound: Option<HashMap<String, Info>>,
    serverbound: Option<HashMap<String, Info>>,
}

#[derive(Deserialize)]
struct Info {
    protocol_id: u8,
}

pub fn impl_packet(ast: &DeriveInput) -> TokenStream {
    let Some(mut packet_name) = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("packet_name"))
        .and_then(|attr| {
            if let Ok(meta) = attr.meta.require_name_value()
                && let Expr::Lit(expr_lit) = &meta.value
                && let Lit::Str(lit_str) = &expr_lit.lit
            {
                Some(lit_str.value())
            } else {
                None
            }
        })
        .or_else(|| camel_case_to_snake_case(&ast.ident.to_string()))
    else {
        return syn::Error::new(
            ast.ident.span(),
            r#"expected UpperCamelCase or add #[packet_name = "<name>"]"#,
        )
        .to_compile_error()
        .into();
    };

    if !packet_name.contains(':') {
        packet_name = format!("minecraft:{packet_name}");
    }

    let Some(protocol_id) = find_protocol_id(&packet_name) else {
        return syn::Error::new(
            ast.ident.span(),
            format!("Couldn't find packet with name `{packet_name}`"),
        )
        .to_compile_error()
        .into();
    };
    let generics = &ast.generics;
    let ident = &ast.ident;

    quote! {
        impl<#generics> crate::protocol::packets::PacketId for #ident {
            const ID: u8 = #protocol_id;
            const IDENTIFIER: &str = #packet_name;
        }
    }
    .into()
}

fn find_protocol_id(packet_name: &str) -> Option<u8> {
    let packet_groups: HashMap<String, Group> =
        serde_json::from_str(include_str!("packet/packets.json"))
            .expect("Unexpected format of packets.json file.");

    for group in packet_groups.into_values() {
        for (name, info) in group.clientbound.iter().flatten() {
            if name == packet_name {
                return Some(info.protocol_id);
            }
        }

        for (name, info) in group.serverbound.iter().flatten() {
            if name == packet_name {
                return Some(info.protocol_id);
            }
        }
    }

    None
}

fn camel_case_to_snake_case(input: &str) -> Option<String> {
    if input.chars().next()?.is_ascii_lowercase() || input.contains('_') {
        return None;
    }

    let mut input_parts = Vec::new();
    let mut last_part = String::new();

    for (index, character) in input.char_indices() {
        if character.is_ascii_uppercase() && index != 0 {
            input_parts.push(last_part);
            last_part = String::new();
        }

        last_part.push(character);
    }
    input_parts.push(last_part);

    let mut output = String::new();

    for part in input_parts {
        output.push_str(&format!("{}_", part.to_ascii_lowercase()));
    }

    output
        .pop()
        .expect("Input should've been checked to have at least one character");

    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_case_to_snake_case() {
        let input = "CamelCase";
        assert_eq!(camel_case_to_snake_case(input), Some("camel_case".into()));

        let input = "Upper";
        assert_eq!(camel_case_to_snake_case(input), Some("upper".into()));

        let input = "U";
        assert_eq!(camel_case_to_snake_case(input), Some("u".into()));

        let input = "UpperLongCamelCase";
        assert_eq!(
            camel_case_to_snake_case(input),
            Some("upper_long_camel_case".into())
        );

        let input = "HTML";
        assert_eq!(camel_case_to_snake_case(input), Some("h_t_m_l".into()));
    }

    #[test]
    fn test_camel_case_to_snake_case_fails() {
        let input = "camelCase";
        assert_eq!(camel_case_to_snake_case(input), None);

        let input = "snake_Case";
        assert_eq!(camel_case_to_snake_case(input), None);

        let input = "real_snake_case";
        assert_eq!(camel_case_to_snake_case(input), None);
    }
}
