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
        is_option: bool,
    }
    let field_names_and_types = match data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .map(|f| {
                    let name = f.ident.clone().expect("Only named fields are supported");
                    let field_type = f.ty.clone();
                    let name_literal = Literal::string(&name.to_string());

                    let (ty, is_option) = match &field_type {
                        syn::Type::Path(tp) => {
                            if tp.path.segments[0].ident.to_string() == "Option" {
                                let inner_type = tp.path.segments[0].arguments.clone();
                                match inner_type {
                                    syn::PathArguments::AngleBracketed(ab) => {
                                        let inner_type = ab.args[0].clone();
                                        match inner_type {
                                            syn::GenericArgument::Type(ty) => (ty, true),
                                            _ => panic!("Only type arguments are supported"),
                                        }
                                    }
                                    _ => panic!("Only angle bracketed arguments are supported"),
                                }
                            } else {
                                (field_type, false)
                            }
                        }
                        _ => (field_type, false),
                    };

                    FieldInfo {
                        name,
                        ty,
                        name_literal,
                        is_option,
                    }
                })
                .collect::<Vec<_>>(),
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let field_checkers_in_try_from = field_names_and_types.iter().map(|FieldInfo { name, ty, name_literal, is_option }| {
        let value_at_field_name = format_ident!("value_at_{}", name);
        let field_name_is_prelim_for = format_ident!("{}PrelimFor", name);

        if *is_option {
            quote! {
                let #value_at_field_name = <yrs::MapRef as yrs::Map>::get(&map_ref, txn, #name_literal);

                if let Some(value) = #value_at_field_name {
                    #[allow(non_camel_case_types)]
                    type #field_name_is_prelim_for = <#ty as yrs::block::Prelim>::Return;
                    <#field_name_is_prelim_for as yrs_wrappers::try_from_yrs_value::TryFromYrsValue>::try_from_yrs_value(
                        value,
                        txn,
                    )?;
                };
            }
        } else {
            quote! {
                let #value_at_field_name = match <yrs::MapRef as yrs::Map>::get(&map_ref, txn, #name_literal) {
                    Some(value) => value,
                    None => return Err(yrs_wrappers::yrs_wrapper_error::YrsWrapperError::YMapMissingAttr {
                        attr: stringify!(#name).to_string(),
                    }),
                };

                #[allow(non_camel_case_types)]
                type #field_name_is_prelim_for = <#ty as yrs::block::Prelim>::Return;

                <#field_name_is_prelim_for as yrs_wrappers::try_from_yrs_value::TryFromYrsValue>::try_from_yrs_value(
                    #value_at_field_name,
                    txn,
                )?;
            }
        }

    });

    let name_without_prelim = format_ident!("{}", name.to_string().trim_end_matches("Prelim"));
    let main_struct = quote! {
        #[derive(Clone, Debug)]
        pub struct #name_without_prelim(yrs::MapRef);
    };

    let try_from_yrs_value_impl = quote! {
        impl yrs_wrappers::try_from_yrs_value::TryFromYrsValue for #name_without_prelim {

            /// `txtn` IS used in #field_checkers_in_try_from, but the compiler doesn't know that,
            /// apparently.
            #[allow(unused_variables)]
            fn try_from_yrs_value(
                value: yrs::types::Value,
                txn: &impl yrs::ReadTxn,
            ) -> yrs_wrappers::yrs_wrapper_error::YrsResult<Self> {
                let map_ref = <yrs::types::Value as yrs_wrappers::yrs_wrapper_error::UnwrapYrsValue>::unwrap_yrs_map(value)?;

                #(#field_checkers_in_try_from)*

                Ok(#name_without_prelim(map_ref))
            }
        }
    };

    let lines_in_prelim_integrate = field_names_and_types.iter().map(
        |FieldInfo {
             name,
             name_literal,
             is_option,
             ..
         }| {
            let line = quote! {
                <yrs::MapRef as yrs::Map>::insert(&map, txn, #name_literal, #name);
            };

            let line = if *is_option {
                quote! {
                    if let Some(#name) = #name {
                        #line
                    };
                }
            } else {
                line
            };

            quote! {
                let #name = self.#name;
                #line
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
            .map(|FieldInfo { name, name_literal, ty, is_option }| {
                let field_name_is_prelim_for = format_ident!("{}PrelimFor", name);

            let mut return_type = quote! {
                yrs_wrappers::yrs_wrapper_error::YrsResult<<#ty as yrs::block::Prelim>::Return>
            };

            if *is_option {
                return_type = quote! { Option<#return_type> };
            }

            // field_name_is_prelim_for is defined way below, btw.
           
            let body = if *is_option {
                quote! {
                    let yrs_value = <yrs::MapRef as yrs::Map>::get(&self.0, txn, #name_literal)?;

                    // I'm not sure if this will be to expensive.
                    Some(<#field_name_is_prelim_for as yrs_wrappers::try_from_yrs_value::TryFromYrsValue>::try_from_yrs_value(
                        yrs_value, txn,
                    ))
                }
            } else {
                quote! {
                    let yrs_value = <yrs::MapRef as yrs::Map>::get(&self.0, txn, #name_literal).ok_or_else(|| {
                        yrs_wrappers::yrs_wrapper_error::YrsWrapperError::YMapMissingAttr {
                            attr: #name_literal.to_string(),
                        }
                    })?;

                    // I'm not sure if this will be to expensive.
                    <#field_name_is_prelim_for as yrs_wrappers::try_from_yrs_value::TryFromYrsValue>::try_from_yrs_value(
                        yrs_value, txn,
                    )
                }
            };


            quote! {
                pub fn #name(
                    &self,
                    txn: impl std::ops::Deref<Target = impl yrs::ReadTxn>,
                ) -> #return_type {
                    #[allow(non_camel_case_types)]
                    type #field_name_is_prelim_for = <#ty as yrs::block::Prelim>::Return;

                    let txn = txn.deref();

                    #body
                }
            }
            })
            .collect::<Vec<_>>();

    let main_struct_impl = quote! {
        impl #name_without_prelim {
            #(#attr_access_impls)*
        }
    };

    let as_mut_impl = quote! {
        impl core::convert::AsMut<yrs::types::Branch> for #name_without_prelim {
            fn as_mut(&mut self) -> &mut yrs::types::Branch {
                <yrs::MapRef as core::convert::AsMut<yrs::types::Branch>>::as_mut(&mut self.0)
            }
        }
    };

    let observable_impl = quote! {
        impl yrs::types::Observable for #name_without_prelim {

            type Event = <yrs::MapRef as yrs::types::Observable>::Event;

            fn try_observer_mut(
                &mut self
            ) -> Option<&mut yrs::observer::Observer<std::sync::Arc<dyn Fn(&yrs::TransactionMut<'_>, &Self::Event)>>> {
                self.0.try_observer_mut()
            }

            fn try_observer(
                &self
            ) -> Option<&yrs::observer::Observer<std::sync::Arc<dyn Fn(&yrs::TransactionMut<'_>, &Self::Event)>>> {
                self.0.try_observer()
            }

        }
    };

    let calls_in_yrs_display = field_names_and_types.iter().map(|field_name_and_type| {
        let name = &field_name_and_type.name;
        let name_string = name.to_string();

        if field_name_and_type.is_option {
            return quote! {
                if let Some(value) = self.#name(txn) {
                    let value = value?;
                    result = result + #name_string + ": " + &yrs_wrappers::yrs_display::YrsDisplay::fmt(&value, txn)? + ", ";
                } else {
                    result = result + #name_string + ": None, ";
                }
            };
        }
        quote! {
            result = result + #name_string + ": " + &yrs_wrappers::yrs_display::YrsDisplay::fmt(&self.#name(txn)?, txn)? + ", ";
        }
    });
    let yrs_display_impl = quote! {
        impl yrs_wrappers::yrs_display::YrsDisplay for #name_without_prelim {
            fn fmt(&self, txn: &impl yrs::ReadTxn) -> yrs_wrappers::yrs_wrapper_error::YrsResult<String> {
                let mut result = String::new();
                result = result + "{ ";
                #(#calls_in_yrs_display)*
                result = result + " }";
                Ok(result)
            }
        }

    };

    let gen = quote! {
        #main_struct

        #try_from_yrs_value_impl

        #prelim_impl

        #try_from_block_ptr_impl

        #main_struct_impl

        #as_mut_impl

        #observable_impl

        #yrs_display_impl
    };

    gen.into()
}
