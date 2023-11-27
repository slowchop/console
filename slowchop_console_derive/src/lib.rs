extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
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
                                "isize" => OrderedArgument {
                                    wrap_type: WrapType::None,
                                    argument_type: ArgumentType::ISize,
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
                                        "isize" => ArgumentType::ISize,
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
                                        "f32" => ArgumentType::Float32,
                                        "isize" => ArgumentType::ISize,

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

#[derive(Debug, PartialEq)]
enum WrapType {
    None,
    Option,
    Vec,
}

#[derive(Debug)]
enum ArgumentType {
    String,
    Float32,
    ISize,
}

fn actions(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let variants = match &ast.data {
        syn::Data::Enum(data) => &data.variants,
        _ => {
            panic!("Actions can only be derived for enums: {:?}", ast.ident);
        }
    };

    let actions: Vec<Action> = variants.iter().map(|v| Action::from_variant(v)).collect();

    // TODO: Make sure there are no options after required arguments.
    // TODO: Make sure Vec is only the last argument.

    let resove_actions = actions.iter().map(|action| {
        let name_str = &action.name;
        let name_ident = syn::Ident::new(name_str, name_str.span());

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
                // Before we can parse the arguments, we need to know how many there are.

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
                            eprintln!("- R E Q U I R E D -----argument type: {argument_type:#?}");
                            parse_argument_type(argument_type, is_last, &name_str)
                        }
                        WrapType::Option => {
                            has_seen_option = true;
                            eprintln!("- O P T I O N -----argument type: {argument_type:#?}");
                            // parse_argument_type(argument_type, true, is_last, &name_str)

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
                                ArgumentType::Float32 => {
                                    quote! {
                                        iter_args.next().map(|v| {
                                            v.parse().map_err(|err| ::slowchop_console::Error::ParseFloatError(#name_str.to_string(), err))
                                        }).transpose()?
                                    }
                                }
                                ArgumentType::ISize => {
                                    quote! {
                                        iter_args.next().map(|v| {
                                            v.parse().map_err(|err| ::slowchop_console::Error::ParseIntError(#name_str.to_string(), err))
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
                            // parse_argument_type(argument_type, is_last, &name_str)
                            // Iterate over the rest of the arguments, and parse them, depending on the type.

                            match argument_type {
                                ArgumentType::String => {
                                    quote! {
                                        iter_args.map(|s| s.to_string()).collect::<Vec<_>>()
                                    }
                                }
                                ArgumentType::Float32 => {
                                    quote! {
                                        iter_args
                                            .map(|s| s.parse().map_err(|err| ::slowchop_console::Error::ParseFloatError(#name_str.to_string(), err)))
                                            .collect::<Result<Vec<_>, _>>()?
                                    }
                                }
                                ArgumentType::ISize => {
                                    quote! {
                                        iter_args
                                            .map(|s| s.parse().map_err(|err| ::slowchop_console::Error::ParseIntError(#name_str.to_string(), err)))
                                            .collect::<Result<Vec<_>, _>>()?
                                    }
                                }
                            }

                        }
                    };

                    eprintln!("arg: {}", arg.to_string());

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

                    eprintln!("given_args: {}", given_args);
                    eprintln!("required_args: {}", #required_args);
                    eprintln!("optional_args: {}", #optional_args);
                    eprintln!("max_args: {}", #max_args);

                    if !#final_arg_consumes_everything && given_args > #max_args {
                        return Err(::slowchop_console::Error::TooManyArguments(#name_str.to_string()));
                    }

                    Ok(Self::#name_ident(
                        #(#args),*
                    ))
                });
            }
        }

        // eprintln!("tokens: {:#?}", tokens);
        // for token in &tokens {
        //     eprintln!("token: {}", token);
        // }

        quote! {
            #name_str => {
                #(#tokens)*
            }
        }
    });

    eprintln!("resove_actions: {:#?}", resove_actions);

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

                match user_action.as_str() {
                    #(#resove_actions)*
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
                    iter_args.next().ok_or(::slowchop_console::Error::NotEnoughArguments{
                        action: #name_str.to_string(),
                    })?.to_string()
                }
            }
        }
        ArgumentType::Float32 => {
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
        ArgumentType::ISize => {
            quote! {
                iter_args
                    .next()
                    .ok_or(::slowchop_console::Error::NotEnoughArguments {
                        action: #name_str.to_string()
                    })?
                    .parse()
                    .map_err(|err| ::slowchop_console::Error::ParseIntError(#name_str.to_string(), err))?
            }
        }
    }
}
