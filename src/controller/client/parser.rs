use nom::branch::alt;
use nom::bytes::streaming::{escaped_transform, take_while, take_while1};
use nom::character::streaming::{char};
use nom::combinator::recognize;
use nom::Err;
use nom::IResult;
use nom::multi::many1;
use nom::sequence::{delimited, terminated};
use nom::sequence::tuple;

#[derive(Debug, PartialEq)]
pub enum Error {
    Incomplete,
    Error,
}

pub fn parse(input: &str) -> Result<(&str, (String, Vec<String>)), (&str, Error)> {
    match do_parse(input) {
        Ok(result) => Ok(result),
        Err(error) => match error {
            Err::Incomplete(_) => {
                Err((input, Error::Incomplete))
            },
            _ => Err((input, Error::Error)),
        },
    }
}


fn do_parse<'a>(input: &'a str) -> IResult<&'a str, (String, Vec<String>)> {
    let identifier = argument();
    let arguments = many1(argument());
    let query = tuple((identifier, arguments));
    let endline = char('\n');

    terminated(query, endline)(input)
}

fn argument<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, String> {
    let space_before = take_while(|c| c == ' ');
    let space_after = take_while(|c| c == ' ');
    // Parse a simple string like 123.it's_an_example (only space, newline and double quote characters are not allowed).
    let simple_string = |input: &'a str| -> IResult<&'a str, String> {
        let (input, output) = take_while1(|c| c != ' ' && c != '\n' && c != '"')(input)?;

        Ok((input, output.to_string()))
    };
    // An escaped string like "I can contain \\ and \"." (every character is allowed, only backslash and double quote characters must be escaped).
    let normal = take_while1(|c| c != '\\' && c != '"');
    let escaped_string = escaped_transform(
        normal,
        '\\',
        alt((recognize(char('\\')), recognize(char('"')))),
    );
    let delimited_escaped_string = delimited(char('"'), escaped_string, char('"'));
    // A string, formatted as a simple or escaped string.
    let string = alt((simple_string, delimited_escaped_string));

    delimited(space_before, string, space_after)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        // Test basic valid buffers.
        assert_eq!(
            parse("A VERSION\n"),
            Ok(("", (String::from("A"), vec![String::from("VERSION")]))),
        );
        assert_eq!(
            parse("    Id   VERSION toto   32t\\ata titi 111   \n"),
            Ok(("", (String::from("Id"), vec![String::from("VERSION"), String::from("toto"), String::from("32t\\ata"), String::from("titi"), String::from("111")]))),
        );
        assert_eq!(
            parse("\"123\" \"UNSET\"\n"),
            Ok(("", (String::from("123"), vec![String::from("UNSET")]))),
        );
        assert_eq!(
            parse(" \"\\\"\" \"\\\"\"   \n"),
            Ok(("", (String::from("\""), vec![String::from("\"")]))),
        );
        assert_eq!(
            parse("   *$a12  UNSET  \"\n\"  \"I can\\\" con$tain\\\\every.thing\\\"\"  \n"),
            Ok(("", (String::from("*$a12"), vec![String::from("UNSET"), String::from("\n"), String::from(r#"I can" con$tain\every.thing""#)]))),
        );
        assert_eq!(
            parse("A SET app.domain.example_job.0 \"2020-05-26 22:26:18\"\n"),
            Ok(("", (String::from("A"), vec![String::from("SET"), String::from("app.domain.example_job.0"), String::from("2020-05-26 22:26:18")]))),
        );
        // Test invalid buffers.
        assert_eq!(
            parse("\n"),
            Err(("\n", Error::Error)),
        );
        assert_eq!(
            parse("A\n"),
            Err(("A\n", Error::Error)),
        );
        // Test incomplete buffers.
        assert_eq!(
            parse("VER"),
            Err(("VER", Error::Incomplete)),
        );
        assert_eq!(
            parse(" \"\\"),
            Err((" \"\\", Error::Incomplete)),
        );
        // Test valid buffers with more data.
        assert_eq!(
            parse("A VERSION\ntoto\nhey"),
            Ok(("toto\nhey", (String::from("A"), vec![String::from("VERSION")]))),
        );
        assert_eq!(
            parse("  XYZ    UNSET  \"\n\"  \"I can\\\" con$tain\\\\every.thing\\\"\"  \n\n\nHEYHEY \"next"),
            Ok(("\n\nHEYHEY \"next", (String::from("XYZ"), vec![String::from("UNSET"), String::from("\n"), String::from(r#"I can" con$tain\every.thing""#)]))),
        );
    }
}
