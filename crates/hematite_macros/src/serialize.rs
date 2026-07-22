use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Expr, Lit, Member, spanned::Spanned};

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
        Data::Enum(data_enum) => {
            let mut enum_arms = Vec::new();

            for (index, variant) in data_enum.variants.iter().enumerate() {
                if !variant.attrs.is_empty() || !variant.fields.is_empty() {
                    return syn::Error::new(
                        variant.span(),
                        "Variant can't have any fields or attributes",
                    )
                    .into_compile_error()
                    .into();
                }

                let index = match variant
                    .discriminant
                    .as_ref()
                    .map(|(_, discriminant)| match discriminant {
                        Expr::Lit(expr_lit) => match &expr_lit.lit {
                            Lit::Int(lit_int) => lit_int.base10_parse(),
                            _ => Err(syn::Error::new(
                                discriminant.span(),
                                "Discriminant has to be a number literal",
                            )),
                        },
                        _ => Err(syn::Error::new(
                            discriminant.span(),
                            "Discriminant has to be a number literal",
                        )),
                    })
                    .unwrap_or(Ok(index))
                {
                    Ok(index) => index,
                    Err(err) => return err.into_compile_error().into(),
                } as i32;

                let variant_ident = &variant.ident;

                enum_arms.push(quote! {
                    Self::#variant_ident => hematite_serialization::builtin_types::var_int::VarInt(#index),
                });
            }

            quote! {
                let varint = match self {
                    #(#enum_arms)*
                    _ => return Err(hematite_serialization::ser::Error::Syntax),
                };

                varint.serialize(writer)?;
            }
        }
        Data::Union(_) => unimplemented!("Data Unions are not supported"),
    };

    quote! {
        impl hematite_serialization::ser::Serialize for #struct_name {
            fn serialize<W: std::io::prelude::Write>(&self, writer: &mut W) -> Result<(), hematite_serialization::ser::Error> {
                #serialize
                Ok(())
            }
        }
    }
    .into()
}
