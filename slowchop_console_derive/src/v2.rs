use crate::actions;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::collections::HashMap;
use syn::{parse_macro_input, DeriveInput, Fields};

#[derive(Debug)]
struct Action {
    ident: Ident,
    args: Args,
}

#[derive(Debug)]
enum Args {
    Empty,
    Enum(Vec<EnumVariant>),
    // Struct(Vec<StructField>),
    // Optional(Box<Args>),
    // Vec(Box<Args>),
    // String,
}

#[derive(Debug)]
struct EnumVariant {
    ident: Ident,
    fields: Vec<Ident>,
}

#[derive(Debug)]
struct StructField {
    ident: Ident,
    ty: Args,
}

pub fn actions2(ast: &DeriveInput) -> TokenStream {
    let action = parse(&ast);
    dbg!(&action);

    validate(&action);
    generate(&action)
}

fn parse(ast: &DeriveInput) -> Action {
    dbg!(ast);

    let args = match &ast.data {
        syn::Data::Enum(data_enum) => parse_enum(data_enum),
        _ => {
            println!("not enum");
            Args::Empty
        }
    };

    Action {
        ident: ast.ident.clone(),
        args,
    }
}

fn parse_enum(data_enum: &syn::DataEnum) -> Args {
    let mut enum_variants = Vec::new();

    for variant in &data_enum.variants {
        let ident = variant.ident.clone();
        match &variant.fields {
            Fields::Named(_) => {}
            Fields::Unnamed(fields) => {
                let mut found_fields = Vec::new();
                for field in &fields.unnamed {
                    match &field.ty {
                        syn::Type::Path(type_path) => {
                            let field_ident = type_path
                                .path
                                .segments
                                .first()
                                .expect(
                                    format!("Expected a type path, found {:?}", type_path).as_str(),
                                )
                                .ident
                                .clone();

                            found_fields.push(field_ident);
                        }
                        _ => {}
                    }
                }

                enum_variants.push(EnumVariant {
                    ident,
                    fields: found_fields,
                });
            }
            Fields::Unit => {}
        }
    }

    Args::Enum(enum_variants)
}

fn validate(action: &Action) {}

fn generate(action: &Action) -> TokenStream {
    let action_ident = &action.ident;
    dbg!(action_ident);

    // let mut tokens = TokenStream::new();
    let mut tokens = Vec::new();

    match &action.args {
        Args::Enum(variants) => {
            tokens.push(quote! {
                let _enum = 1;
            });

            // We need to parse this "command" (or subcommand).
            tokens.push(quote! {});

            for variant in variants {
                let variant_ident = &variant.ident;
                let fields = &variant.fields;
                for field in fields {
                    tokens.push(quote! {
                        let _field = 1;
                        let x = #field::resolve(s);
                    });
                }
            }
        }
        x => {
            let x = format!("{:?}", x);
            //
            tokens.push(quote! {
                let x = "#x";
            });
        }
    };

    dbg!(&tokens);

    quote! {
        impl ::slowchop_console::ActionsHandler for #action_ident {
            fn resolve(s: &mut str) -> ::std::result::Result<Self, ::slowchop_console::Error> {

                let start = 1;

                #(#tokens)*


                // just to test the macro... i want to access the first enum ident

                // #ident::resolve(s)

                let end = 1;
                todo!()
            }
        }
    }
}
