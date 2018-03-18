use quote::Tokens;
use syn::{Data, DeriveInput, Fields, Ident};
use mul_like::{struct_exprs, tuple_exprs};
use std::iter;
use std::collections::HashSet;
use utils::{get_field_types_iter, named_to_vec, unnamed_to_vec, add_where_clauses_for_new_ident};

pub fn expand(input: &DeriveInput, trait_name: &str) -> Tokens {
    let trait_ident = Ident::from(trait_name);
    let trait_path = &quote!(::std::ops::#trait_ident);
    let method_name = trait_name.to_string();
    let method_name = method_name.trim_right_matches("Assign");
    let method_name = method_name.to_lowercase();
    let method_ident = Ident::from(method_name.to_string() + "_assign");
    let input_type = &input.ident;

    let (exprs, fields) = match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Unnamed(ref fields) => {
                let field_vec = unnamed_to_vec(fields);
                (tuple_exprs(&field_vec, &method_ident), field_vec)
            }
            Fields::Named(ref fields) => {
                let field_vec = named_to_vec(fields);
                (struct_exprs(&field_vec, &method_ident), field_vec)
            }
            _ => panic!(format!("Unit structs cannot use derive({})", trait_name)),
        },

        _ => panic!(format!("Only structs can use derive({})", trait_name)),
    };

    let scalar_ident = &Ident::from("__RhsT");
    let tys: &HashSet<_> = &get_field_types_iter(&fields).collect();
    let scalar_iter = iter::repeat(scalar_ident);
    let trait_path_iter = iter::repeat(trait_path);

    let type_where_clauses = quote!{
        where #(#tys: #trait_path_iter<#scalar_iter>),*
    };

    let new_generics = add_where_clauses_for_new_ident(&input.generics, &fields, scalar_ident, type_where_clauses);
    let (impl_generics, _, where_clause) = new_generics.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();

    quote!(
        impl#impl_generics #trait_path<#scalar_ident> for #input_type#ty_generics #where_clause{
            #[inline]
            fn #method_ident(&mut self, rhs: #scalar_ident#ty_generics) {
                #(#exprs;
                  )*
            }
        }
    )
}
