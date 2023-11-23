/*
#[derive(Commands)]
enum Console {
    // /quit
    // /quit? -- will show the doc comment
    Quit,

    // /echo sup, how's it going?
    // Note the spaces without quotes. If String is the last type, it will handle no quotes.
    Echo(String),

    // /query Cat -Enemy +Transform
    // Prefix symbols are just implementation details for querying bevy (& With, ! Without, ? Optional, * Changed, + Added, - Removed, etc )
    Query(Vec<String>),

    // /complex-query component Apple without Transform

    // /fixed-first-then-vec 1 "string 1" "string 2" "string 3"
    FixedFirstThenVec(i32, Vec<String>),

    // Won't compile because ambiguous
    // Query(Vec<f32>, Vec<String>),

    // /two-strings "string 1" string 2
    // /two-strings "string 1" "string 2"
    // Again, last string can be free of quotes when it has spaces.
    TwoStrings(String, String),

    // /two-floats 1.2 3.5
    TwoFloats(f32, f64),

    // /optional-second first
    // /optional-second first second
    OptionalSecond(String, Option<String>),
    // This won't compile because it becomes ambiguous
    // ErrorThis(Option<String>, String),

    // /spawn Ant
    // Spawn(Thing),

}

// enum Thing {
//     Apple,
//     Ant,
//     Banana,
// }

// This is how the macro would generate the code
// impl Commands {
//     pub fn resolve(s: &str) -> Result<Self, crate::Error> {
//         let mut iter = s.split_whitespace();
//         let command = iter.next()?;
//         let args = iter.collect::<Vec<_>>();
//
//         match command {
//             "quit" => Some(Commands::Quit),
//             "echo" => Some(Commands::Echo(args.join(" "))),
//             "query" => Some(Commands::Query(args.iter().map(|s| s.to_string()).collect())),
//             "fixed-first-then-vec" => {
//                 let first = args[0].parse().ok()?;
//                 let rest = args[1..].iter().map(|s| s.to_string()).collect();
//                 Some(Commands::FixedFirstThenVec(first, rest))
//             },
//             "two-strings" => Some(Commands::TwoStrings(args[0].to_string(), args[1].to_string())),
//             "two-floats" => {
//                 let a = args[0].parse().ok()?;
//                 let b = args[1].parse().ok()?;
//                 Some(Commands::TwoFloats(a, b))
//             },
//             "optional-second" => Some(Commands::OptionalSecond(args[0].to_string(), args.get(1).map(|s| s.to_string()))),
//             _ => None,
//         }
//     }
// }

// struct StructIdea {
//     a: String,
//     b: Option<f32>,
//     c: bool,
// }
//
// struct AnotherStruct {
//     d: usize,
// }
//
// struct DeepStruct {
//     struct_idea: StructIdea,
//     another_struct: AnotherStruct,
// }
//
// enum QueryArg {
//     Component(String),
//     With(String),
//     Without(String),
// }

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Vec3;

    #[test]
    fn brainstorm() {
        assert_eq!(Console::resolve("quit"), Ok(Commands::Quit));
        assert_eq!(Console::resolve("echo sup"), Ok(Console::Echo("echo sup".to_string())));
        assert_eq!(Console::resolve(r#""echo sup""#), Ok(Console::Echo("echo sup".to_string())));
    }
}

 */
extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, GenericArgument, PathArguments};

#[proc_macro_derive(Commands)]
pub fn commands_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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

        if variant.fields.len() == 0 {
            return Action {
                name,
                action_type: ActionType::NoArgs,
            };
        }

        let action_type = match &variant.fields {
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
            _ => panic!("Unknown fields: {:?}", variant.fields),
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

    let found: Vec<Action> = vec![];

    for variant in variants.iter() {
        let action = Action::from_variant(variant);
        eprintln!("action: {:?}", action);
    }

    // let gen = quote! {
    //     impl #name {
    //         pub fn resolve(s: &str) -> ::std::result::Result<Self, ::slowchop_console::Error> {
    //             let mut iter = s.split_whitespace();
    //             let command = iter.next().ok_or_else(|| ::slowchop_console::Error::NoCommandGiven)?;
    //             let args = iter.collect::<Vec<_>>();
    //
    //             match command {
    //                 #(#collected => Ok(#name::#variants),)*
    //                 _ => Err(::slowchop_console::Error::UnknownCommand(command.to_string())),
    //             }
    //         }
    //     }
    // };
    // gen.into()

    TokenStream::new()
}
