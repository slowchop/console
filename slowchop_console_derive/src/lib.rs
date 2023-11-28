extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, GenericArgument, PathArguments};

#[proc_macro_derive(Actions)]
pub fn actions_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let output: TokenStream = actions(&input);
    proc_macro::TokenStream::from(output)
}

#[derive(Debug)]
struct Action {
    name: String,
    ident: syn::Ident,
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
                                "String" => OrderedArgument::none(ArgumentType::String),
                                "f32" | "f64" => OrderedArgument::none(ArgumentType::Float),
                                "usize" | "isize" | "u8" | "i8" | "u16" | "i16" | "u32" | "i32"
                                | "u64" | "i64" | "u128" | "i128" => {
                                    OrderedArgument::none(ArgumentType::Integer)
                                }

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
                                        "f32" | "f64" => ArgumentType::Float,
                                        "isize" | "usize" => ArgumentType::Integer,
                                        _ => panic!("Unknown path type inside option: {:?}", ident),
                                    };

                                    OrderedArgument {
                                        wrap_type: WrapType::Option,
                                        argument_type,
                                    }
                                }
                                "Vec" => {
                                    // We just want to handle "String", "f32", etc within the Vec.
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
                                        "f32" | "f64" => ArgumentType::Float,
                                        "usize" | "isize" | "u8" | "i8" | "u16" | "i16" | "u32"
                                        | "i32" | "u64" | "i64" | "u128" | "i128" => {
                                            ArgumentType::Integer
                                        }
                                        _ => panic!("Unknown path type inside vec: {:?}", ident),
                                    };

                                    OrderedArgument {
                                        wrap_type: WrapType::Vec,
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

        Action {
            name,
            ident: variant.ident.clone(),
            action_type,
        }
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

impl OrderedArgument {
    fn none(argument_type: ArgumentType) -> Self {
        Self {
            wrap_type: WrapType::None,
            argument_type,
        }
    }
}

#[derive(Debug, PartialEq)]
enum WrapType {
    None,
    Option,
    Vec,
}

#[derive(Debug)]
enum ArgumentType {
    // Char,
    String,
    Integer,
    // Bool,
    Float,
}

fn actions(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let variants = match &ast.data {
        syn::Data::Enum(data) => &data.variants,
        _ => {
            panic!("Actions can only be derived for enums: {:?}", name);
        }
    };

    let actions: Vec<Action> = variants.iter().map(|v| Action::from_variant(v)).collect();

    // TODO: Make sure there are no options after required arguments.
    // TODO: Make sure Vec is only the last argument.

    let resolve_actions = actions.iter().map(|action| {
        let name_str = &action.name;
        let name_ident = &action.ident;

        let mut tokens = vec![];
        match &action.action_type {
            ActionType::NoArgs => {
                tokens.push(quote! {
                    if iter_args.len() > 0 {
                        return Err(::slowchop_console::Error::TooManyArguments(#name_str.to_string()));
                    }

                    Ok(Self::#name_ident)
                });
            }
            ActionType::OrderedArgs(ordered_args) => {
                let required_args = ordered_args.iter().filter(|arg| arg.wrap_type == WrapType::None).count();
                let optional_args = ordered_args.iter().filter(|arg| arg.wrap_type == WrapType::Option).count();
                let max_args = required_args + optional_args;

                let mut final_arg_consumes_everything = false;

                let mut args: Vec<TokenStream> = vec![];
                let mut has_seen_option = false;

                for (idx, arg) in ordered_args.iter().enumerate() {
                    let is_last = idx == ordered_args.len() - 1;

                    let argument_type = &arg.argument_type;
                    let wrap_type = &arg.wrap_type;

                    let arg = match wrap_type {
                        WrapType::None => {
                            if has_seen_option {
                                panic!("Required arguments must come before optional arguments: {:?}", ordered_args);
                            }
                            parse_argument_type(argument_type, is_last, &name_str)
                        }
                        WrapType::Option => {
                            has_seen_option = true;
                            match argument_type {
                                ArgumentType::String => {
                                    if is_last {
                                        final_arg_consumes_everything = true;
                                        quote! {
                                            if iter_args.len() == 0 {
                                                None
                                            } else {
                                                Some(iter_args.map(|s| s.to_string()).collect::<Vec<_>>().join(" "))
                                            }
                                        }

                                    } else {
                                        quote! {
                                            iter_args.next().map(|v| v.to_string())
                                        }
                                    }
                                }
                                ArgumentType::Integer => {
                                    quote! {
                                        iter_args.next().map(|v| {
                                            v.parse().map_err(|err| ::slowchop_console::Error::ParseIntError(#name_str.to_string(), err))
                                        }).transpose()?
                                    }
                                }
                                ArgumentType::Float => {
                                    quote! {
                                        iter_args.next().map(|v| {
                                            v.parse().map_err(|err| ::slowchop_console::Error::ParseFloatError(#name_str.to_string(), err))
                                        }).transpose()?
                                    }
                                }
                            }

                        }
                        WrapType::Vec => {
                            if !is_last {
                                panic!("Vec can only be the last argument: {:?}", ordered_args);
                            }
                            final_arg_consumes_everything = true;

                            match argument_type {
                                ArgumentType::String => {
                                    quote! {
                                        iter_args.map(|s| s.to_string()).collect::<Vec<_>>()
                                    }
                                }
                                ArgumentType::Integer => {
                                    quote! {
                                        iter_args
                                            .map(|s| s.parse().map_err(|err| ::slowchop_console::Error::ParseIntError(#name_str.to_string(), err)))
                                            .collect::<Result<Vec<_>, _>>()?
                                    }
                                }
                                ArgumentType::Float => {
                                    quote! {
                                        iter_args
                                            .map(|s| s.parse().map_err(|err| ::slowchop_console::Error::ParseFloatError(#name_str.to_string(), err)))
                                            .collect::<Result<Vec<_>, _>>()?
                                    }
                                }
                            }

                        }
                    };

                    if let ArgumentType::String = argument_type {
                        if let WrapType::None = wrap_type {
                            if is_last {
                                final_arg_consumes_everything = true;
                            }
                        }
                    }

                    args.push(arg);
                }

                tokens.push(quote! {
                    let given_args = iter_args.len();

                    if !#final_arg_consumes_everything && given_args > #max_args {
                        return Err(::slowchop_console::Error::TooManyArguments(#name_str.to_string()));
                    }

                    Ok(Self::#name_ident(
                        #(#args),*
                    ))
                });
            }
        }

        let name_str = name_str.to_lowercase();

        quote! {
            #name_str => {
                #(#tokens)*
            }
        }
    });

    let gen = quote! {
        impl ::slowchop_console::ActionsImpl for #name {
            /// Resolve the given string into a command.
            fn resolve(s: &str) -> ::std::result::Result<Self, ::slowchop_console::Error> {
                let items = shlex::split(s).unwrap();
                if items.len() == 0 {
                    return Err(::slowchop_console::Error::NoCommandGiven);
                }

                let user_action = &items[0];
                let user_args = &items[1..];
                let mut iter_args = user_args.iter();

                match user_action.to_lowercase().as_str() {
                    #(#resolve_actions)*
                    _ => Err(::slowchop_console::Error::UnknownAction {
                        action: user_action.to_string()
                    }),
                }

            }
        }
    };

    gen.into()
}

fn parse_argument_type(argument_type: &ArgumentType, is_last: bool, name_str: &str) -> TokenStream {
    match argument_type {
        ArgumentType::String => {
            if is_last {
                // Get all remaining arguments and join them.
                quote! {
                    iter_args.map(|s| s.to_string()).collect::<Vec<_>>().join(" ")
                }
            } else {
                quote! {
                    iter_args.next().ok_or(::slowchop_console::Error::NotEnoughArguments {
                        action: #name_str.to_string(),
                    })?.to_string()
                }
            }
        }
        ArgumentType::Integer => {
            quote! {
                iter_args
                    .next()
                    .ok_or(::slowchop_console::Error::NotEnoughArguments{
                        action: #name_str.to_string()
                    })?
                    .parse()
                    .map_err(|err| ::slowchop_console::Error::ParseIntError(#name_str.to_string(), err))?
            }
        }
        ArgumentType::Float => {
            quote! {
                iter_args
                    .next()
                    .ok_or(::slowchop_console::Error::NotEnoughArguments{
                        action: #name_str.to_string()
                    })?
                    .parse()
                    .map_err(|err| ::slowchop_console::Error::ParseFloatError(#name_str.to_string(), err))?
            }
        }
    }
}
