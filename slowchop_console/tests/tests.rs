use slowchop_console::{Actions, ActionsHandler, Error};

#[test]
fn resolve_unit() -> Result<(), Error> {
    assert_eq!(Con::resolve("Quit")?, Con::Quit);
    assert!(Con::resolve("Quit hmm").is_err());

    Ok(())
}

#[test]
fn resolve_single_string() -> Result<(), Error> {
    assert_eq!(Con::resolve("Echo sup")?, Con::Echo("sup".into()));
    assert_eq!(Con::resolve("Echo \"1 2 \"")?, Con::Echo("1 2 ".into()));
    assert_eq!(Con::resolve("Echo 1 2")?, Con::Echo("1 2".into()));
    // TODO: Should this be an error, or just let it join the two last quotes as one?
    // Right now these strings will just concatenate with no error.
    assert_eq!(
        Con::resolve("Echo \"1 2\" \"3\"")?,
        Con::Echo("1 2 3".into())
    );

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
fn resolve_int_types() -> Result<(), Error> {
    assert_eq!(
        Con::resolve("LotsOfDifferentIntTypes 1 2 3 -4 5 -600 7 -80000 9 -10000000000 11 -12")?,
        Con::LotsOfDifferentIntTypes(1, 2, 3, -4, 5, -600, 7, -80000, 9, -10000000000, 11, -12)
    );

    Ok(())
}

#[test]
fn resolve_bool() -> Result<(), Error> {
    assert_eq!(Con::resolve("Bool t")?, Con::Bool(true));
    assert_eq!(Con::resolve("Bool true")?, Con::Bool(true));
    assert_eq!(Con::resolve("Bool True")?, Con::Bool(true));
    assert_eq!(Con::resolve("Bool y")?, Con::Bool(true));
    assert_eq!(Con::resolve("Bool yes")?, Con::Bool(true));
    assert_eq!(Con::resolve("Bool 1")?, Con::Bool(true));

    assert_eq!(Con::resolve("Bool f")?, Con::Bool(false));
    assert_eq!(Con::resolve("Bool false")?, Con::Bool(false));
    assert_eq!(Con::resolve("Bool FALSE")?, Con::Bool(false));
    assert_eq!(Con::resolve("Bool n")?, Con::Bool(false));
    assert_eq!(Con::resolve("Bool no")?, Con::Bool(false));
    assert_eq!(Con::resolve("Bool 0")?, Con::Bool(false));

    Ok(())
}

#[test]
fn vec() -> Result<(), Error> {
    assert_eq!(
        Con::resolve("VecString a b 'c c'")?,
        Con::VecString(vec!["a".to_string(), "b".to_string(), "c c".to_string()])
    );

    assert_eq!(
        Con::resolve("VecFloat32 1.1 2.2 3.3")?,
        Con::VecFloat32(vec![1.1, 2.2, 3.3])
    );

    assert_eq!(
        Con::resolve("VecISize 1 2 3")?,
        Con::VecISize(vec![1, 2, 3])
    );

    Ok(())
}

#[test]
fn optional_float() -> Result<(), Error> {
    assert!(Con::resolve("OptionalFloat asdf").is_err());
    assert_eq!(Con::resolve("OptionalFloat")?, Con::OptionalFloat(None));
    assert_eq!(
        Con::resolve("OptionalFloat 1.2")?,
        Con::OptionalFloat(Some(1.2))
    );

    Ok(())
}

#[test]
fn two_optional_floats() -> Result<(), Error> {
    assert!(Con::resolve("TwoOptionalFloats asdf").is_err());
    assert_eq!(
        Con::resolve("TwoOptionalFloats")?,
        Con::TwoOptionalFloats(None, None)
    );
    assert_eq!(
        Con::resolve("TwoOptionalFloats 1.2")?,
        Con::TwoOptionalFloats(Some(1.2), None)
    );
    assert_eq!(
        Con::resolve("TwoOptionalFloats 1.2 3.4")?,
        Con::TwoOptionalFloats(Some(1.2), Some("3.4".into()))
    );

    Ok(())
}

#[test]
fn required_then_optional() -> Result<(), Error> {
    assert!(Con::resolve("RequiredThenOptional").is_err());
    assert_eq!(
        Con::resolve("RequiredThenOptional 'hi there'")?,
        Con::RequiredThenOptional("hi there".into(), None)
    );
    assert_eq!(
        Con::resolve("RequiredThenOptional 'hi there' 'string 2'")?,
        Con::RequiredThenOptional("hi there".into(), Some("string 2".into()))
    );

    // Second argument without quotes should eat the rest of the words.
    assert_eq!(
        Con::resolve("RequiredThenOptional 'hi there' 1 2 3 4")?,
        Con::RequiredThenOptional("hi there".into(), Some("1 2 3 4".into()))
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
    TwoFloats(f32, f64),
    LotsOfDifferentIntTypes(
        isize,
        usize,
        u8,
        i8,
        u16,
        i16,
        u32,
        i32,
        u64,
        i64,
        u128,
        i128,
    ),
    Bool(bool),
    VecString(Vec<String>),
    VecFloat32(Vec<f32>),
    VecISize(Vec<isize>),
    OptionalFloat(Option<f32>),
    TwoOptionalFloats(Option<f32>, Option<String>),
    RequiredThenOptional(String, Option<String>),
    // Concat(String, Vec<String>),

    // TODO: ordered struct: Value { key: String, set_value: Option<String> }
    // TODO: ordered struct: Concat { separator: String, strings: Vec<String> }
}
