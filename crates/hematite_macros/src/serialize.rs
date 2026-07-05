use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Member};

pub fn impl_serialize_macro(ast: &DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;
    let serialize = match &ast.data {
        Data::Struct(data_struct) => {
            let mut serializables = Vec::new();

            for (field_index, field) in data_struct.fields.iter().enumerate() {
                serializables.push(match field.ident.as_ref() {
                    Some(ident) => quote! {
                        self.#ident.serialize(writer)?;
                    },
                    None => {
                        let member = Member::from(field_index);

                        quote! {
                            self.#member.serialize(writer)?;
                        }
                    }
                });
            }

            quote! {
                #(#serializables)*
            }
        }
        Data::Enum(_data_enum) => todo!("Enums are currently not supported"),
        Data::Union(_) => unimplemented!("Data Unions are not supported"),
    };

    quote! {
        impl crate::protocol::ser_de::ser::Serialize for #struct_name {
            fn serialize<W: std::io::prelude::Write>(&self, writer: &mut W) -> Result<(), crate::protocol::ser_de::ser::Error> {
                #serialize
                Ok(())
            }
        }
    }
    .into()
}
