use crate::Error;
use winnow::ascii::{alpha1, alphanumeric1, escaped_transform, space0};
use winnow::combinator::{alt, cut_err, not, preceded, repeat, terminated};
use winnow::token::{any, none_of, take_until, take_while};
use winnow::{PResult, Parser};

/// Parse an action from the input string, moving forward the input string's pointer, to past the
/// next whitespace.
fn parse_action<'s>(s: &mut &'s str) -> PResult<&'s str> {
    terminated(alphanumeric1, space0).parse_next(s)
}

/// Allow a string with quotes to be parsed, and return the string without the quotes.
///
/// Also allows for escaped quotes.
///
/// # Examples
/// * hello -> hello
/// * "hello world" -> hello
/// * "hello \"world\"" -> hello "world"
/// * 'hello "world"' -> hello "world"
fn parse_string(s: &mut &str) -> PResult<String> {
    terminated(
        alt((
            //
            quote_string('"'),
            quote_string('\''),
            string_no_space,
        )),
        space0,
    )
    .parse_next(s)
}

fn string_no_space(s: &mut &str) -> PResult<String> {
    repeat(1.., none_of(' '))
        .fold(
            || String::new(),
            |mut acc, c| {
                acc.push(c);
                acc
            },
        )
        .parse_next(s)
}

fn quote_string<'s>(quote: char) -> impl Fn(&mut &'s str) -> PResult<String> {
    move |s: &mut &'s str| {
        preceded(
            quote,
            cut_err(terminated(
                repeat(0.., quote_char(quote)).fold(
                    || String::new(),
                    |mut acc, c| {
                        acc.push(c);
                        acc
                    },
                ),
                quote,
            )),
        )
        .parse_next(s)
    }
}

fn quote_char<'s>(quote: char) -> impl Fn(&mut &'s str) -> PResult<char> {
    move |s: &mut &'s str| {
        let c = none_of(quote).parse_next(s)?;
        if c == '\\' {
            any.verify_map(|c| {
                if c == quote || c == '\\' {
                    Some(c)
                } else {
                    None
                }
            })
            .parse_next(s)
        } else {
            Ok(c)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_action() {
        let test_cases = vec![
            ("action nothing", Ok("action"), "nothing"),
            ("action ", Ok("action"), ""),
            ("action", Ok("action"), ""),
        ];

        for (mut s, expected_action, expected_remaining) in test_cases {
            let s = &mut s;
            assert_eq!(parse_action.parse_next(s), expected_action);
            assert_eq!(*s, expected_remaining);
        }
    }

    #[test]
    fn test_parse_string() {
        let test_cases = vec![
            (r#"hello"#, "hello", "", true),
            // No quotes so you can't have spaces.
            (r#"hello world"#, "hello", "world", true),
            (r#""hello world" !"#, "hello world", "!", true),
            (r#""hello \"world\"""#, r#"hello "world""#, "", true),
            (r#"'hello "world\'s"' !"#, r#"hello "world's""#, "!", true),
            // Empty strings must be quoted.
            (r#""#, r#""#, "", false),
            (r#"'' !"#, r#""#, "!", true),
            (r#""" !"#, r#""#, "!", true),
        ];

        for fixture in test_cases {
            let (s, expected, new_pointer, ok) = &fixture;
            let mut s = *s;
            let r = parse_string.parse_next(&mut s);
            if *ok {
                assert_eq!(r, Ok(expected.to_string()), "fixture: {:?}", fixture);
            } else {
                assert!(r.is_err(), "fixture: {:?}", fixture);
            }
            assert_eq!(s, *new_pointer, "fixture: {:?}", fixture);
        }
    }
}
