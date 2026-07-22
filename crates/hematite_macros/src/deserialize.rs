use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{Data, DeriveInput, spanned::Spanned};

pub fn impl_deserialize_macro(ast: &DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;
    let deserialize = match &ast.data {
        Data::Struct(data_struct) => {
            let mut deserializables = Vec::new();
            let mut constructor = Vec::new();

            let mut is_tuple_struct = false;

            for (field_index, field) in data_struct.fields.iter().enumerate() {
                let tuple_ident = Ident::new(format!("member{field_index}").as_str(), field.span());
                let ident = field.ident.as_ref().unwrap_or_else(|| {
                    is_tuple_struct = true;
                    &tuple_ident
                });

                deserializables.push(quote! {
                    let #ident = hematite_serialization::de::Deserialize::deserialize(reader)?;
                });

                constructor.push(quote! {
                    #ident
                });
            }

            if is_tuple_struct {
                quote! {
                    #(#deserializables)*
                    Ok(Self(#(#constructor)*))
                }
            } else {
                quote! {
                    #(#deserializables)*
                    Ok(Self { #(#constructor, )* })
                }
            }
        }
        Data::Enum(_data_enum) => todo!("Enums are currently not supported"),
        Data::Union(_) => unimplemented!("Data Unions are not supported"),
    };

    quote! {
        impl hematite_serialization::de::Deserialize for #struct_name {
            fn deserialize<R: std::io::prelude::BufRead>(reader: &mut R) -> Result<Self, hematite_serialization::de::Error> {
                #deserialize
            }
        }
    }
    .into()
}
