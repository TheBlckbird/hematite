use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn impl_serialize_macro(ast: &DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;
    let serialize = match &ast.data {
        syn::Data::Struct(data_struct) => {
            let mut serializables = Vec::new();

            for field in &data_struct.fields {
                let ident = field.ident.as_ref().unwrap();

                serializables.push(quote! {
                    self.#ident.serialize(writer)?;
                });
            }

            quote! {
                #(#serializables)*
            }
        }
        syn::Data::Enum(data_enum) => todo!("Enums are currently not supported"),
        syn::Data::Union(data_union) => todo!("Data Unions are currently not supported"),
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
