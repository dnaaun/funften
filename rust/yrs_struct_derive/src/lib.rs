extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(YrsStruct, attributes(yrs_value_type))]
pub fn yrs_wrapper_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let data = input.data;

    if !name.to_string().ends_with("Prelim") {
        panic!("To emphasize that the struct that YrsWrapper annotates should be the prelim, the struct name should end with Prelim");
    }

    struct FieldInfo {
        name: syn::Ident,
        ty: syn::Type,
        name_literal: Literal,
    }
    let field_names_and_types = match data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .map(|f| {
                    let field_name = f.ident.clone().expect("Only named fields are supported");
                    let field_type = f.ty.clone();
                    let field_name_literal = Literal::string(&field_name.to_string());

                    FieldInfo {
                        name: field_name,
                        ty: field_type,
                        name_literal: field_name_literal,
                    }
                })
                .collect::<Vec<_>>(),
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let field_checkers_in_try_from = field_names_and_types.iter().map(|FieldInfo { name, ty, name_literal }| {
        let value_at_field_name = format_ident!("value_at_{}", name);
        let field_name_is_prelim_for = format_ident!("{}PrelimFor", name);
        quote! {
            let #value_at_field_name = match <yrs::MapRef as yrs::Map>::get(&map_ref, txn, #name_literal) {
                Some(value) => value,
                None => return Err(yrs_wrappers::yrs_struct::YrsStructDeserializeError::MissingAttribute {
                    attr: stringify!(#name).to_string(),
                }),
            };

            #[allow(non_camel_case_types)]
            type #field_name_is_prelim_for = <#ty as yrs::block::Prelim>::Return;

            let #name = match <#field_name_is_prelim_for as yrs_wrappers::try_from_yrs_value::TryFromYrsValue>::try_from_yrs_value(
                #value_at_field_name,
                txn,
            ) {
                Ok(value) => value,
                Err(err) => return Err(yrs_wrappers::yrs_struct::YrsStructDeserializeError::ElementDeserialize {
                    attr: #name_literal.to_string(),
                    err: Box::new(err)
                }),
            };
        }

    });

    let name_without_prelim = format_ident!("{}", name.to_string().trim_end_matches("Prelim"));
    let main_struct = quote! {
        pub struct #name_without_prelim(yrs::MapRef);
    };

    let try_from_yrs_value_impl = quote! {
        impl yrs_wrappers::try_from_yrs_value::TryFromYrsValue for #name_without_prelim {
            type Error = yrs_wrappers::yrs_struct::YrsStructDeserializeError;

            /// `txtn` IS used in #field_checkers_in_try_from, but the compiler doesn't know that,
            /// apparently.
            #[allow(unused_variables)]
            fn try_from_yrs_value(
                value: yrs::types::Value,
                txn: &yrs::Transaction,
            ) -> Result<Self, Self::Error> {
                let map_ref = match value {
                    yrs::types::Value::YMap(map_ref) => map_ref,
                    _ => return Err(yrs_wrappers::yrs_struct::YrsStructDeserializeError::ExpectedYMap),
                };

                #(#field_checkers_in_try_from)*

                Ok(#name_without_prelim(map_ref))
            }
        }
    };

    let lines_in_prelim_integrate = field_names_and_types.iter().map(
        |FieldInfo {
             name, name_literal, ..
         }| {
            quote! {
                <yrs::MapRef as yrs::Map>::insert(&map, txn, #name_literal, self.#name);
            }
        },
    );

    let prelim_impl = quote! {
        impl yrs::block::Prelim for #name {
            type Return = #name_without_prelim;

            /// Copied from impl Prelim for MapRef.
            fn into_content(
                self,
                txn: &mut yrs::TransactionMut,
            ) -> (yrs::block::ItemContent, Option<Self>) {
                let inner = yrs::types::Branch::new(yrs::types::TYPE_REFS_MAP, None);
                (yrs::block::ItemContent::Type(inner), Some(self))
            }

            fn integrate(self, txn: &mut yrs::TransactionMut, inner_ref: yrs::types::BranchPtr) {
                let map = yrs::MapRef::from(inner_ref);

                #(#lines_in_prelim_integrate)*
            }
        }
    };

    let try_from_block_ptr_impl = quote! {
        impl TryFrom<yrs::block::BlockPtr> for #name_without_prelim {
            type Error = <yrs::MapRef as TryFrom<yrs::block::BlockPtr>>::Error;

            fn try_from(value: yrs::block::BlockPtr) -> Result<Self, Self::Error> {
                let map_ref: yrs::MapRef = value.try_into()?;

                Ok(#name_without_prelim(map_ref))
            }
        }
    };

    let attr_access_impls =
        field_names_and_types
            .iter()
            .map(|FieldInfo { name, name_literal, ty }| {
                let field_name_is_prelim_for = format_ident!("{}PrelimFor", name);
                quote! {
    pub fn #name(
        &self,
        txn: &yrs::Transaction,
    ) -> Result<
        <#ty as yrs::block::Prelim>::Return,
        yrs_wrappers::yrs_struct::YrsStructDeserializeError,
    > {
        let yrs_value = <yrs::MapRef as yrs::Map>::get(&self.0, txn, #name_literal).ok_or_else(|| {
            yrs_wrappers::yrs_struct::YrsStructDeserializeError::MissingAttribute {
                attr: #name_literal.to_string(),
            }
        })?;

        #[allow(non_camel_case_types)]
        type #field_name_is_prelim_for = <#ty as yrs::block::Prelim>::Return;

        // I'm not sure if this will be to expensive.
        let deserialized = <#field_name_is_prelim_for as yrs_wrappers::try_from_yrs_value::TryFromYrsValue>::try_from_yrs_value(
            yrs_value, txn,
        );

        deserialized.map_err(|err| {
            yrs_wrappers::yrs_struct::YrsStructDeserializeError::ElementDeserialize {
                attr: #name_literal.to_string(),
                err: Box::new(err),
            }
        })
    }
                }
            })
            .collect::<Vec<_>>();

    let main_struct_impl = quote! {
        impl #name_without_prelim {
            #(#attr_access_impls)*
        }
    };

    let gen = quote! {
        #main_struct

        #try_from_yrs_value_impl

        #prelim_impl

        #try_from_block_ptr_impl

        #main_struct_impl
    };

    gen.into()
}
