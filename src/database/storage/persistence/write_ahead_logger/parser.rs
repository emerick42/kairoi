use nom::combinator::flat_map;
use nom::IResult;
use nom::number::streaming::be_u32;
use nom::bytes::streaming::take;
use nom::Err as NomErr;

pub type Entry = Vec<u8>;
pub struct Parser {}
#[derive(Debug, PartialEq)]
pub enum ParseError<'a> {
    CorruptedContent,
    Incomplete(Vec<Entry>, &'a [u8]),
}
pub type ParseResult<'a> = Result<Vec<Entry>, ParseError<'a>>;

impl Parser {
    /// Create a new parser.
    pub fn new() -> Self {
        Self {}
    }

    /// Parse the given input, returning a collection of entries. If the input is incomplete, it
    /// returns a ParseError::Incomplete containing the collection of entries that have been parsed
    /// and the input left. If the input is invalid, it returns a ParseError::CorruptedContent.
    pub fn parse<'a>(&self, input: &'a [u8]) -> ParseResult<'a> {
        let mut entries = Vec::new();
        let mut input = input;

        while input.len() > 0 {
            match self.parse_entry(input) {
                Ok((input_left, entry)) => {
                    entries.push(entry);
                    input = input_left;
                },
                Err(error) => match error {
                    NomErr::Incomplete(_) => return Err(ParseError::Incomplete(entries, input)),
                    _ => return Err(ParseError::CorruptedContent),
                },
            };
        };

        Ok(entries)
    }

    fn parse_entry<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], Entry> {
        let entry = |input: &'a [u8]| -> IResult<&'a [u8], Entry> {
            let (input, output) = flat_map(be_u32, take)(input)?;

            Ok((input, output.to_vec()))
        };

        entry(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let parser = Parser::new();

        // Test basic valid buffers.
        assert_eq!(
            parser.parse(&vec![0, 0, 0, 1, 0]),
            Ok(vec![vec![0]]),
        );
        assert_eq!(
            parser.parse(&vec![0, 0, 0, 1, 0, 0, 0, 0, 1, 1]),
            Ok(vec![vec![0], vec![1]]),
        );
        assert_eq!(
            parser.parse(&vec![0, 0, 0, 8, 0, 1, 2, 3, 4, 5, 6, 7]),
            Ok(vec![vec![0, 1, 2, 3, 4, 5, 6, 7]]),
        );
        // Test incomplete buffers.
        assert_eq!(
            parser.parse(&vec![0]),
            Err(ParseError::Incomplete(vec![], &vec![0])),
        );
        assert_eq!(
            parser.parse(&vec![0, 0, 0, 1]),
            Err(ParseError::Incomplete(vec![], &vec![0, 0, 0, 1])),
        );
        assert_eq!(
            parser.parse(&vec![0, 0, 0, 8, 0, 1, 2, 3]),
            Err(ParseError::Incomplete(vec![], &vec![0, 0, 0, 8, 0, 1, 2, 3])),
        );
        assert_eq!(
            parser.parse(&vec![0, 0, 0, 1, 0, 0, 0, 0, 2]),
            Err(ParseError::Incomplete(vec![vec![0]], &vec![0, 0, 0, 2])),
        );
    }
}
