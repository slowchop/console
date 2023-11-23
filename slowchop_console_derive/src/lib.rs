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
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Commands)]
pub fn commands_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // let input = proc_macro2::TokenStream::from(input);
    let input = parse_macro_input!(input as DeriveInput);

    let output: proc_macro2::TokenStream = commands(&input);

    proc_macro::TokenStream::from(output)
}

fn commands(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let variants = match &ast.data {
        syn::Data::Enum(data) => &data.variants,
        _ => {
            panic!("Commands can only be derived for enums: {:?}", ast.ident);
        }
    };

    let mut collected = vec![];

    for variant in variants {
        let variant_name = &variant.ident; // e.g. Echo

        // variant.fields.iter().for_each(|field| {
        //     let field_name = &field.ident;
        //     let field_type = &field.ty;
        //
        //     match field_type {
        //         syn::Type::Path(path) => {
        //             let path = &path.path;
        //             let path = quote! { #path };
        //             if path.to_string() == "String" {
        //                 todo!("String: {name} / {variant_name} / {field_name:?} / {field_type:?}/ {path:?}")
        //             } else {
        //                 todo!("Path: {name} / {variant_name} / {field_name:?} / {field_type:?}")
        //             }
        //         }
        //         _ => todo!("Field Type: {name} / {variant_name} / {field_name:?} / {field_type:?}"),
        //     }
        // });

        let variant_name = variant_name.to_string();
        let variant_name = variant_name.to_lowercase();

        collected.push(variant_name);
    }

    let gen = quote! {
        impl #name {
            pub fn resolve(s: &str) -> ::std::result::Result<Self, ::slowchop_console::Error> {
                let mut iter = s.split_whitespace();
                let command = iter.next().ok_or_else(|| ::slowchop_console::Error::NoCommandGiven)?;
                let args = iter.collect::<Vec<_>>();

                match command {
                    #(#collected => Ok(#name::#variants),)*
                    _ => Err(::slowchop_console::Error::UnknownCommand(command.to_string())),
                }
            }
        }
    };

    gen.into()
}
