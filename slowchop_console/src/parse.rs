use crate::Error;
use winnow::ascii::{alphanumeric1, space0};
use winnow::combinator::{preceded, repeat, terminated};
use winnow::token::{take_until, take_while};
use winnow::{PResult, Parser};

/// Parse an action from the input string, moving forward the input string's pointer, to past the
/// next whitespace.
fn parse_action<'s>(s: &mut &'s str) -> PResult<&'s str> {
    let v = terminated(alphanumeric1, space0).parse_next(s)?;

    Ok(v)
}

fn discard_whitespace1(s: &mut str) -> PResult<()> {
    todo!()
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
}
