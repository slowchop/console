use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{DeriveInput, Fields};

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
        let mut found_fields = Vec::new();

        match &variant.fields {
            Fields::Named(_) => {}
            Fields::Unnamed(fields) => {
                // let mut found_fields = Vec::new();
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
            }
            Fields::Unit => {}
        }

        enum_variants.push(EnumVariant {
            ident,
            fields: found_fields,
        });
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
            generate_enum(&mut tokens, action, variants);
        }
        x => {
            let x = format!("{:?}", x);
            //
            tokens.push(quote! {
                let xxxxxx = "#x";
            });
        }
    };

    quote! {
        impl ::slowchop_console::ActionsHandler for #action_ident {
            fn resolve(s: &mut &str) -> ::std::result::Result<Self, ::slowchop_console::Error> {

                let start = 1;

                #(#tokens)*

                let end = 1;
                todo!()
            }
        }
    }
}

fn generate_enum(tokens: &mut Vec<TokenStream>, action: &Action, variants: &[EnumVariant]) {
    tokens.push(quote! {
        let _enum = 1;
    });

    // We need to parse this "command" (or subcommand).
    tokens.push(quote! {
        let action = ::slowchop_console::parse::action(s)?;
        println!("action: {:?}", action);
    });

    let mut inner_tokens = Vec::new();
    for variant in variants {
        let variant_tokens = generate_enum_variant(action, variant);
        inner_tokens.extend(variant_tokens);
    }

    tokens.push(quote! {
        match action {
            #(#inner_tokens)*
            _ => {
                return Err(::slowchop_console::Error::UnknownAction);
            }
        };
    });
}

fn generate_enum_variant(action: &Action, variant: &EnumVariant) -> Vec<TokenStream> {
    let mut tokens = Vec::new();

    let variant_ident = &variant.ident;
    let variant_name = variant_ident.to_string();
    let fields = &variant.fields;

    tokens.push(quote! {
        #variant_name => {
            let _variant = 1;
        }
    });

    // for field in fields {
    //     let field = field.to_string();
    //     tokens.push(quote! {
    //         #variant_name => {
    //             let _field = #field;
    //             // let _field = #field;
    //             // let x = #field::resolve(s);
    //         }
    //     });
    // }

    tokens
}
