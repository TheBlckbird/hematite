use std::collections::HashMap;

use itertools::{Either, Itertools};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    Ident, LitStr, Token, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

use crate::packet::Packets;

struct AllPackets {
    states: HashMap<Ident, Group>,
}

impl Parse for AllPackets {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut states = HashMap::new();

        while let Ok(state_ident) = input.parse::<Ident>() {
            let is_networking_side = if input.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in input);
                let flag: Ident = content.parse()?;

                if flag != "networking" {
                    return Err(syn::Error::new(flag.span(), "expected `networking`"));
                }

                true
            } else {
                false
            };

            input.parse::<Token![:]>()?;
            let mut directions = HashMap::new();

            while let Ok(direction) = input.parse::<Direction>() {
                let content;
                syn::braced!(content in input);
                let entries: Punctuated<PacketEntry, Token![,]> =
                    Punctuated::parse_terminated(&content)?;

                directions.insert(direction, entries.into_iter().collect());
            }

            input.parse::<Token![;]>()?;

            let group = Group {
                is_networking_side,
                directions,
            };

            states.insert(state_ident, group);
        }

        Ok(AllPackets { states })
    }
}

struct Group {
    is_networking_side: bool,
    directions: HashMap<Direction, Vec<PacketEntry>>,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Direction {
    Clientbound,
    Serverbound,
}

impl Parse for Direction {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        match ident.to_string().as_str() {
            "CB" => Ok(Self::Clientbound),
            "SB" => Ok(Self::Serverbound),
            _ => Err(syn::Error::new(ident.span(), "Expected either CB or SB")),
        }
    }
}

struct PacketEntry {
    ident: Ident,
    internal_name: Option<LitStr>,
}

impl Parse for PacketEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let packet = input.parse()?;

        if input.parse::<Token![=]>().is_err() {
            return Ok(Self {
                ident: packet,
                internal_name: None,
            });
        }

        Ok(Self {
            ident: packet,
            internal_name: input.parse()?,
        })
    }
}

struct PacketWithMetadata {
    ident: Ident,
    internal_name: String,
    id: u8,
    state: Ident,
    direction: Direction,
    is_networking_layer: bool,
}

impl PacketWithMetadata {
    fn new(
        ident: Ident,
        internal_name: String,
        id: u8,
        state: Ident,
        direction: Direction,
        is_networking_layer: bool,
    ) -> Self {
        Self {
            ident,
            internal_name,
            id,
            state,
            direction,
            is_networking_layer,
        }
    }
}

impl TryFrom<AllPackets> for Vec<PacketWithMetadata> {
    type Error = TokenStream;

    fn try_from(value: AllPackets) -> Result<Self, Self::Error> {
        let mut all_packets = Vec::new();

        for (state, group) in value.states.iter() {
            for (direction, packet_group) in &group.directions {
                for packet in packet_group {
                    let Some(internal_ident) = packet
                        .internal_name
                        .as_ref()
                        .map(|internal_name| internal_name.value())
                        .or_else(|| camel_case_to_snake_case(&packet.ident.to_string()))
                    else {
                        let span = match &packet.internal_name {
                            Some(internal_name) => internal_name.span(),
                            None => packet.ident.span(),
                        };

                        return Err(syn::Error::new(
                            span,
                            format!("{} isn't a valid packet name", packet.ident),
                        )
                        .into_compile_error()
                        .into());
                    };

                    let internal_name = if internal_ident.contains(':') {
                        internal_ident.clone()
                    } else {
                        format!("minecraft:{internal_ident}")
                    };

                    let Some(id) = find_protocol_id(&internal_name, direction) else {
                        let span = match &packet.internal_name {
                            Some(internal_name) => internal_name.span(),
                            None => packet.ident.span(),
                        };

                        return Err(syn::Error::new(
                            span,
                            format!("{} isn't a known packet", internal_ident),
                        )
                        .into_compile_error()
                        .into());
                    };

                    all_packets.push(PacketWithMetadata::new(
                        packet.ident.clone(),
                        internal_ident,
                        id,
                        state.clone(),
                        *direction,
                        group.is_networking_side,
                    ));
                }
            }
        }

        Ok(all_packets)
    }
}

pub fn impl_all_packets(input: TokenStream) -> TokenStream {
    let all_packets: AllPackets = parse_macro_input!(input as AllPackets);

    let mut server_states = Vec::new();

    for state in all_packets.states.keys() {
        server_states.push(quote! {
            #state,
        });
    }

    let packets_list: Vec<PacketWithMetadata> = match all_packets.try_into() {
        Ok(packets_list) => packets_list,
        Err(error) => return error,
    };

    let (all_clientbound_packets, all_serverbound_packets): (Vec<_>, Vec<_>) = packets_list
        .iter()
        .partition_map(|packet| match packet.direction {
            Direction::Clientbound => Either::Left(packet),
            Direction::Serverbound => Either::Right(packet),
        });

    let (networking_clientbound, engine_clientbound): (Vec<_>, Vec<_>) =
        all_clientbound_packets.iter().partition_map(|packet| {
            let packet_ident = &packet.ident;
            let generated = quote! {
                #packet_ident(#packet_ident),
            };

            match packet.is_networking_layer {
                true => Either::Left(generated),
                false => Either::Right(generated),
            }
        });

    let (networking_serverbound, engine_serverbound): (Vec<_>, Vec<_>) =
        all_serverbound_packets.iter().partition_map(|packet| {
            let packet_ident = &packet.ident;
            let generated = quote! {
                #packet_ident(#packet_ident),
            };

            match packet.is_networking_layer {
                true => Either::Left(generated),
                false => Either::Right(generated),
            }
        });

    let mut get_ids = Vec::new();
    let mut engine_serializables = Vec::new();
    let mut from_ids = Vec::new();
    let mut send_events_args = Vec::new();
    let mut send_events = Vec::new();

    let engine_serverbound_ident = Ident::new("EngineSBPackets", Span::call_site());
    let engine_clientbound_ident = Ident::new("EngineCBPackets", Span::call_site());
    let networking_serverbound_ident = Ident::new("NetworkingSBPackets", Span::call_site());
    let networking_clientbound_ident = Ident::new("NetworkingCBPackets", Span::call_site());

    for packet in packets_list.iter() {
        let packet_ident = &packet.ident;
        let id = &packet.id;
        let internal_name = Ident::new(&packet.internal_name, packet.ident.span());

        match packet.direction {
            Direction::Clientbound => {
                if packet.is_networking_layer {
                    get_ids.push(quote! {
                        Self::Networking(#networking_clientbound_ident::#packet_ident(_)) => #id,
                    });

                    engine_serializables.push(quote! {
                        Self::Networking(#networking_clientbound_ident::#packet_ident(#internal_name)) => hematite_serialization::ser::Serialize::serialize(#internal_name, writer),
                    });
                } else {
                    get_ids.push(quote! {
                        Self::Engine(#engine_clientbound_ident::#packet_ident(_)) => #id,
                    });

                    engine_serializables.push(quote! {
                        Self::Engine(#engine_clientbound_ident::#packet_ident(#internal_name)) => hematite_serialization::ser::Serialize::serialize(#internal_name, writer),
                    });
                }
            }

            Direction::Serverbound => {
                let state = &packet.state;
                let internal_name_writer = Ident::new(
                    format!("{internal_name}_writer").as_str(),
                    internal_name.span(),
                );

                if packet.is_networking_layer {
                    from_ids.push(quote! {
                        (#id, ServerState::#state) => Some(
                            hematite_serialization::de::Deserialize::deserialize(reader)
                                .map(#networking_serverbound_ident::#packet_ident)
                                .map(Self::Networking)
                        ),
                    });
                } else {
                    from_ids.push(quote! {
                        (#id, ServerState::#state) => Some(
                            hematite_serialization::de::Deserialize::deserialize(reader)
                                .map(#engine_serverbound_ident::#packet_ident)
                                .map(Self::Engine)
                        ),
                    });

                    send_events_args.push(quote! {
                        mut #internal_name_writer: bevy_ecs::message::MessageWriter<#packet_ident>,
                    });

                    send_events.push(quote! {
                        Self::#packet_ident(#internal_name) => {#internal_name_writer.write(#internal_name);}
                    });
                }
            }
        }
    }

    quote! {
        pub enum ServerState {
            #(#server_states)*
        }

        pub enum RoutedCBPacket {
            Networking(#networking_clientbound_ident),
            Engine(#engine_clientbound_ident),
        }

        impl RoutedCBPacket {
            pub fn get_id(&self) -> u8 {
                match self {
                    #(#get_ids)*
                }
            }

            pub fn serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<(), hematite_serialization::ser::Error> {
                #[allow(non_snake_case)]
                match self {
                    #(#engine_serializables)*
                }
            }
        }

        pub enum #engine_clientbound_ident {
            #(#engine_clientbound)*
        }

        pub enum #networking_clientbound_ident {
            #(#networking_clientbound)*
        }

        pub enum RoutedSBPacket {
            Networking(#networking_serverbound_ident),
            Engine(#engine_serverbound_ident),
        }

        impl RoutedSBPacket {
            pub fn from_id<R: std::io::BufRead>(
                id: &u8,
                server_state: &ServerState,
                reader: &mut R,
            ) -> Option<Result<Self, hematite_serialization::de::Error>> {
                match (id, server_state) {
                    #(#from_ids)*
                    _ => None,
                }
            }
        }

        pub enum #engine_serverbound_ident {
            #(#engine_serverbound)*
        }

        impl #engine_serverbound_ident {
            /// Can be run as a one shot system:
            ///
            /// ```rust
            /// commands.run_system_cached_with(EngineSBPackets::send_event, incoming_packet);
            /// ```
            #[allow(clippy::too_many_arguments)]
            pub fn send_event(
                bevy_ecs::system::In(packet): bevy_ecs::system::In<Self>,
                #(#send_events_args)*
            ) {
                match packet {
                    #(#send_events)*
                }
            }
        }

        pub enum #networking_serverbound_ident {
            #(#networking_serverbound)*
        }
    }
    .into()
}

fn find_protocol_id(packet_name: &str, direction: &Direction) -> Option<u8> {
    let packet_states: Packets = serde_json::from_str(include_str!("packet/packets.json"))
        .expect("Unexpected format of packets.json file.");

    for sides in packet_states.into_values() {
        let direction_packets = match direction {
            Direction::Clientbound => sides.clientbound,
            Direction::Serverbound => sides.serverbound,
        };

        for (name, packet) in direction_packets.iter().flatten() {
            if name == packet_name {
                return Some(packet.protocol_id);
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
