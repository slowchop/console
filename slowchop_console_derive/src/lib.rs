extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, GenericArgument, PathArguments, PathSegment};

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

                            if let Some(argument_type) = ArgumentType::from_ident(ident) {
                                OrderedArgument::none(argument_type)
                            } else {
                                match ident.to_string().as_str() {
                                    "Option" => OrderedArgument {
                                        wrap_type: WrapType::Option,
                                        argument_type: ArgumentType::from_inner(segment, "Option"),
                                    },
                                    "Vec" => OrderedArgument {
                                        wrap_type: WrapType::Vec,
                                        argument_type: ArgumentType::from_inner(segment, "Vec"),
                                    },
                                    _ => panic!("Unknown path type: {:?}", ident),
                                }
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
    Bool,
    Float,
}

impl ArgumentType {
    fn from_ident(ident: &syn::Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "String" => Some(ArgumentType::String),
            "f32" | "f64" => Some(ArgumentType::Float),
            "usize" | "isize" | "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64"
            | "u128" | "i128" => Some(ArgumentType::Integer),
            "bool" => Some(ArgumentType::Bool),
            _ => None,
        }
    }

    fn from_inner(segment: &PathSegment, section: &str) -> Self {
        let PathArguments::AngleBracketed(bracketed) = &segment.arguments else {
            panic!(
                "Expected angle bracketed arguments in {section}: {:?}",
                segment.arguments
            );
        };
        let arg = &bracketed.args[0];
        let GenericArgument::Type(ty) = arg else {
            panic!("Expected type in {section}: {arg:?}");
        };
        let syn::Type::Path(path) = ty else {
            panic!("Expected path in {section}: {ty:?}");
        };
        let segment = path.path.segments.last().unwrap();
        let ident = &segment.ident;

        ArgumentType::from_ident(ident)
            .unwrap_or_else(|| panic!("Unknown path type in {section}: {ident:?}"))
    }
}

fn actions(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let variants = match &ast.data {
        syn::Data::Enum(data) => &data.variants,
        _ => {
            panic!("Actions can only be derived for enums: {:?}", name);
        }
    };

    let actions: Vec<Action> = variants.iter().map(Action::from_variant).collect();

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
                            parse_argument_type(argument_type, is_last, name_str)
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
                                ArgumentType::Bool => {
                                    quote! {
                                        iter_args.next().map(|v| {
                                            ::slowchop_console::parse_bool(v)
                                                .ok_or(::slowchop_console::Error::ParseBoolError(#name_str.to_string()))
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
                                ArgumentType::Bool => {
                                    quote! {
                                        iter_args
                                            .map(|s| ::slowchop_console::parse_bool(s)
                                                .ok_or(::slowchop_console::Error::ParseBoolError(#name_str.to_string())))
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

    quote! {
        impl ::slowchop_console::ActionsHandler for #name {
            fn resolve(s: &str) -> ::std::result::Result<Self, ::slowchop_console::Error> {
                let items = shlex::split(s).unwrap();
                if items.len() == 0 {
                    return Err(::slowchop_console::Error::NoActionGiven);
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
    }
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
        ArgumentType::Bool => {
            quote! {
                ::slowchop_console::parse_bool(
                    iter_args
                        .next()
                        .ok_or(::slowchop_console::Error::NotEnoughArguments {
                            action: #name_str.to_string()
                        })?
                ).ok_or(::slowchop_console::Error::ParseBoolError(#name_str.to_string()))?
            }
        }
    }
}
