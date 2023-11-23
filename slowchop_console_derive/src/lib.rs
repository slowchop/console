extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, GenericArgument, PathArguments};

#[proc_macro_derive(Actions)]
pub fn actions_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let output: TokenStream = commands(&input);

    proc_macro::TokenStream::from(output)
}

#[derive(Debug)]
struct Action {
    name: String,
    action_type: ActionType,
}

impl Action {
    fn from_variant(variant: &syn::Variant) -> Action {
        let name = variant.ident.to_string();

        let action_type = match &variant.fields {
            syn::Fields::Unit => ActionType::NoArgs,
            syn::Fields::Unnamed(fields) => {
                let mut ordered_args = vec![];

                for field in fields.unnamed.iter() {
                    let ordered_argument = match &field.ty {
                        syn::Type::Path(path) => {
                            let path = &path.path;
                            let segment = path.segments.last().unwrap();
                            let ident = &segment.ident;

                            match ident.to_string().as_str() {
                                "String" => OrderedArgument {
                                    wrap_type: WrapType::None,
                                    argument_type: ArgumentType::String,
                                },
                                "f32" => OrderedArgument {
                                    wrap_type: WrapType::None,
                                    argument_type: ArgumentType::Float32,
                                },
                                "Option" => {
                                    // We just want to handle "String", "f32", etc within the Option.
                                    let PathArguments::AngleBracketed(bracketed) =
                                        &segment.arguments
                                    else {
                                        panic!(
                                            "Expected angle bracketed arguments: {:?}",
                                            segment.arguments
                                        );
                                    };
                                    let arg = &bracketed.args[0];
                                    let GenericArgument::Type(ty) = arg else {
                                        panic!("Expected type: {:?}", arg);
                                    };
                                    let syn::Type::Path(path) = ty else {
                                        panic!("Expected path: {:?}", ty);
                                    };
                                    let segment = path.path.segments.last().unwrap();
                                    let ident = &segment.ident;

                                    let argument_type = match ident.to_string().as_str() {
                                        "String" => ArgumentType::String,
                                        "f32" => ArgumentType::Float32,
                                        _ => panic!("Unknown path type: {:?}", ident),
                                    };

                                    OrderedArgument {
                                        wrap_type: WrapType::Option,
                                        argument_type,
                                    }
                                }
                                _ => panic!("Unknown path type: {:?}", ident),
                            }
                        }
                        _ => panic!("Unknown argument_type: {:?}", field.ty),
                    };

                    ordered_args.push(ordered_argument);
                }

                ActionType::OrderedArgs(ordered_args)
            }
            _ => {
                panic!("Unknown fields: {:?}", variant.fields);
            }
        };

        Action { name, action_type }
    }
}

#[derive(Debug)]
enum ActionType {
    NoArgs,
    OrderedArgs(Vec<OrderedArgument>),
}

#[derive(Debug)]
struct OrderedArgument {
    wrap_type: WrapType,
    argument_type: ArgumentType,
}

#[derive(Debug)]
enum WrapType {
    None,
    Option,
    Vec,
}

#[derive(Debug)]
enum ArgumentType {
    String,
    Float32,
}

fn commands(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let variants = match &ast.data {
        syn::Data::Enum(data) => &data.variants,
        _ => {
            panic!("Commands can only be derived for enums: {:?}", ast.ident);
        }
    };

    let actions: Vec<Action> = variants.iter().map(|v| Action::from_variant(v)).collect();

    // TODO: Make sure there are no options after required arguments.
    // TODO: Make sure Vec is only the last argument.

    eprintln!("actions: {:#?}", actions);

    let action_quotes = actions.iter().map(|action| {
        let name = &action.name;
        let name = syn::Ident::new(name, name.span());

        let mut tokens = vec![];
        match &action.action_type {
            ActionType::NoArgs => {
                tokens.push(quote! {
                    Commands::#name
                });
            }
            ActionType::OrderedArgs(ordered_args) => {
                // Output like this, e.g. for this
                // TwoArgs(1, 2)
                /*

                Ok(Commands::TwoArgs({
                    let arg_str = iter.next().ok_or(Error::NotEnoughArguments(name.to_string()))?;
                    arg_str.parse().map_err(|err| Error::Conversion(name.to_string(), err))?
                }, {
                    let arg_str = iter.next().ok_or(Error::NotEnoughArguments(name.to_string()))?;
                    arg_str.parse().map_err(|err| Error::Conversion(name.to_string(), err))?
                }))
                 */

                let mut args: Vec<TokenStream> = vec![];

                for arg in ordered_args {
                    let argument_type = &arg.argument_type;
                    let wrap_type = &arg.wrap_type;

                    // let arg_name = syn::Ident::new("arg_str", name.span());

                    let arg = match argument_type {
                        ArgumentType::String => {
                            quote! {
                                iter.next().unwrap().to_string()
                            }
                        }
                        ArgumentType::Float32 => {
                            quote! {
                                // #arg_name.parse().map_err(|err| Error::Conversion(name.to_string(), err))?
                                todo!()
                            }
                        }
                    };

                    let arg = match wrap_type {
                        WrapType::None => {
                            quote! {
                                #arg
                            }
                        }
                        WrapType::Option => {
                            quote! {
                                Some(#arg)
                            }
                        }
                        WrapType::Vec => {
                            quote! {
                                vec![#arg]
                            }
                        }
                    };

                    args.push(arg);
                }

                tokens.push(quote! {
                    Commands::#name
                    (
                        #(#args),*
                    )
                });
            }
        }

        quote! {
            #name => {
                #(#tokens)*
            }
        }
    });

    let gen = quote! {
        impl #name {
            /// Resolve the given string into a command.
            pub fn resolve(s: &str) -> ::std::result::Result<Self, ::slowchop_console::Error> {
                let items = shlex::split(s).unwrap();
                if items.len() == 0 {
                    return Err(::slowchop_console::Error::NoCommandGiven);
                }

                let user_action = &items[0];
                let rest = &items[1..];
                let mut iter = rest.iter();

                match user_action {
                    #(#action_quotes)*
                    _ => Err(::slowchop_console::Error::UnknownCommand(user_action.to_string())),
                }

            }
        }
    };

    gen.into()
}
