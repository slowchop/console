use crate::Error;
use winnow::ascii::{alpha1, alphanumeric1, escaped_transform, space0};
use winnow::combinator::{alt, cut_err, preceded, repeat, terminated};
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
fn parse_string<'s>(s: &mut &'s str) -> PResult<String> {
    alt((
        // string starting with double quotes
        double_quote_string,
        // string starting with single quotes
        // single_quote_string,
        // string with no quotes
        // take_until(1.., space0),
    ))
    .parse_next(s)
}

fn double_quote_string<'s>(s: &mut &'s str) -> PResult<String> {
    let quote = '"';

    preceded(
        quote,
        cut_err(terminated(
            repeat(0.., double_quote_char).fold(
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

fn double_quote_char<'s>(s: &mut &'s str) -> PResult<char> {
    let c = none_of('\"').parse_next(s)?;
    if c == '\\' {
        any.verify_map(|c| {
            Some(match c {
                '"' | '\\' => c,
                _ => return None,
            })
        })
        .parse_next(s)
    } else {
        Ok(c)
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
            (r#""hello world""#, Ok("hello world")),
            (r#""hello \"world\"""#, Ok(r#"hello "world""#)),
            (r#"'hello "world\'s"'"#, Ok(r#"hello "world's""#)),
        ];

        for (s, expected) in test_cases {
            let mut s = s;
            assert_eq!(parse_string.parse_next(&mut s), expected.map(String::from));
        }
    }
}
