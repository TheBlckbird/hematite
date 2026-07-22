use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{Data, DeriveInput, Expr, Lit, spanned::Spanned};

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
                    #index => Ok(Self::#variant_ident),
                });
            }

            quote! {
                let variant: hematite_serialization::builtin_types::var_int::VarInt = hematite_serialization::de::Deserialize::deserialize(reader)?;

                match *variant {
                    #(#enum_arms)*
                    _ => Err(hematite_serialization::de::Error::Syntax),
                }
            }
        }
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
