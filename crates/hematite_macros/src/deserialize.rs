use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn impl_deserialize_macro(ast: &DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;
    let deserialize = match &ast.data {
        syn::Data::Struct(data_struct) => {
            let mut deserializables = Vec::new();
            let mut constructor = Vec::new();

            for field in &data_struct.fields {
                let ident = field.ident.as_ref().unwrap();

                deserializables.push(quote! {
                    let #ident = crate::protocol::ser_de::de::Deserialize::deserialize(reader)?;
                });

                constructor.push(quote! {
                    #ident,
                });
            }

            quote! {
                #(#deserializables)*
                Ok(Self { #(#constructor)* })
            }
        }
        syn::Data::Enum(data_enum) => todo!(),
        syn::Data::Union(data_union) => todo!(),
    };

    quote! {
        impl crate::protocol::ser_de::de::Deserialize for #struct_name {
            fn deserialize<R: std::io::prelude::BufRead>(reader: &mut R) -> Result<Self, crate::protocol::ser_de::de::Error> {
                #deserialize
            }
        }
    }
    .into()
}
