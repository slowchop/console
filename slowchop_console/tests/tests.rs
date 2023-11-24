use slowchop_console::{Actions, Error};

// // TODO: #[derive(Actions)]
// enum Console {
//     // /quit
//     // /quit? -- will show the doc comment
//     Quit,
//
//     // /echo sup, how's it going?
//     // Note the spaces without quotes. If String is the last type, it will handle no quotes.
//     Echo(String),
//
//     // /query Cat -Enemy +Transform
//     // Prefix symbols are just implementation details for querying bevy (& With, ! Without, ? Optional, * Changed, + Added, - Removed, etc )
//     Query(Vec<String>),
//
//     // /complex-query component Apple without Transform
//
//     // /fixed-first-then-vec 1 "string 1" "string 2" "string 3"
//     FixedFirstThenVec(i32, Vec<String>),
//
//     // Won't compile because ambiguous
//     // Query(Vec<f32>, Vec<String>),
//
//     // /two-strings "string 1" string 2
//     // /two-strings "string 1" "string 2"
//     // Again, last string can be free of quotes when it has spaces.
//     TwoStrings(String, String),
//
//     // /two-floats 1.2 3.5
//     TwoFloats(f32, f64),
//
//     // /optional-second first
//     // /optional-second first second
//     OptionalSecond(String, Option<String>),
//     // This won't compile because it becomes ambiguous
//     // ErrorThis(Option<String>, String),
//
//     // Bevy types (that are Reflect?)
//     // /spawn thing x=2.2 y=5.5 z=-9
//     // Spawn(Thing, Vec3),
//
//     // /struct-idea a="hello there" b=2 c=true
//     // /struct-idea a= c=false
//     // /struct-idea a=sup c=on
//     // bool can be 1 on yes true (is this a good idea?)
//     // StructIdea(StructIdea),
//
//     // ------ multiple levels ----- ??
//     // do they flatten so you can join multiple structs together??
//     // JoinStructs(StructIdea, AnotherStruct),
//     // this is complicated
//     // DeepStructs(DeepStruct),
// }

// enum Thing {
//     Apple,
//     Ant,
//     Banana,
// }

// This is how the macro would generate the code
// impl Console {
//     pub fn resolve(s: &str) -> Option<Self> {
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

#[test]
fn resolve_unit() -> Result<(), Error> {
    assert_eq!(Con::resolve("Quit")?, Con::Quit);
    assert!(Con::resolve("Quit hmm").is_err());

    Ok(())
}

#[test]
fn resolve_single_string() -> Result<(), Error> {
    // assert_eq!(Con::resolve("Echo sup")?, Con::Echo("sup".into()));
    // assert_eq!(Con::resolve("Echo \"1 2 \"")?, Con::Echo("1 2 ".into()));
    assert_eq!(Con::resolve("Echo 1 2")?, Con::Echo("1 2".into()));
    // // TODO: Should this be an error, or just let it join the two last quotes as one?
    // assert_eq!(
    //     Con::resolve("Echo \"1 2\" \"3\"")?,
    //     Con::Echo("1 2 3".into())
    // );

    Ok(())
}

#[test]
fn resolve_two_floats() -> Result<(), Error> {
    assert_eq!(Con::resolve("TwoFloats 1.2 3.5")?, Con::TwoFloats(1.2, 3.5));
    assert_eq!(Con::resolve("TwoFloats 1 -5")?, Con::TwoFloats(1., -5.));

    // Extra argument
    assert!(Con::resolve("TwoFloats 1.2 3.5 5.5").is_err());

    Ok(())
}

#[test]
fn array() -> Result<(), Error> {
    assert_eq!(
        Con::resolve("VecISize 1 2 3")?,
        Con::VecISize(vec![1, 2, 3])
    );

    Ok(())
}

#[test]
fn complete() {
    // assert_eq!(Commands::complete("qu"), vec!["quit", "query"]);
    // assert_eq!(Commands::complete("spawn a"), vec!["pple", "nt"]);
}

#[derive(Debug, PartialEq, Actions)]
enum Con {
    Quit,
    Echo(String),
    TwoStrings(String, String),
    TwoFloats(f32, f32),
    VecISize(Vec<isize>),
    //
    // /// Set or Get
    // // TODO: ordered struct: Value { key: String, set_value: Option<String> }
    // GetOrSet(String, Option<String>),
    //
    // // TODO: ordered struct: Concat { separator: String, strings: Vec<String> }
    // Concat(String, Vec<String>),
}
