use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, char, one_of},
    combinator::{map_res, recognize},
    multi::{many0, many1},
    sequence::{pair, preceded, terminated},
    IResult,
};

pub mod control_file {
    use nom::{
        branch::alt,
        bytes::complete::{tag, take_till, take_until, take_while},
        character::{
            self,
            complete::{alphanumeric1, crlf, line_ending, not_line_ending, one_of, space0, space1},
        },
        combinator::{consumed, eof, map, not, opt, recognize, value},
        error::context,
        multi::{many0, many1},
        sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
        IResult, Parser,
    };

    pub fn field_name(input: &str) -> IResult<&str, &str> {
        context(
            "field name",
            terminated(
                recognize(pair(alphanumeric1, many0(alt((alphanumeric1, tag("-")))))),
                character::complete::char(':'),
            ),
        )(input)
    }

    pub fn my_non_line_ending(input: &str) -> IResult<&str, &str> {
        take_while(|c| c != '\n' && c != '\r')(input)
    }
    pub fn end_of_line_or_string(input: &str) -> IResult<&str, &str> {
        alt((eof, line_ending))(input)
    }
    pub fn line(input: &str) -> IResult<&str, &str> {
        recognize(pair(my_non_line_ending, end_of_line_or_string))(input)
    }

    pub fn paragraph(input: &str) -> IResult<&str, Vec<&str>> {
        many0(recognize(pair(not_line_ending, end_of_line_or_string)))(input)
    }

    pub fn rest_of_line(input: &str) -> IResult<&str, &str> {
        recognize(pair(opt(not_line_ending), end_of_line_or_string))(input)
    }
    pub fn continuation_line(input: &str) -> IResult<&str, &str> {
        recognize(tuple((space1, my_non_line_ending, end_of_line_or_string)))(input)
    }

    pub fn field_pair(input: &str) -> IResult<&str, (&str, &str)> {
        separated_pair(
            field_name,
            space0,
            recognize(pair(rest_of_line, many0(continuation_line))),
        )(input)
    }

    /// creates a parser for a field name
    pub fn specific_field_name<'a>(
        name: &'static str,
    ) -> impl Parser<&'a str, (), nom::error::Error<&'a str>> {
        value((), field_name.and_then(tag(name)))
    }

    /// creates a parser for a single-line field with a specific name.
    ///
    /// The created parser returns the value, including the trailing line ending.
    pub fn named_single_line_field<'a>(
        name: &'static str,
    ) -> impl Parser<&'a str, &'a str, nom::error::Error<&'a str>> {
        preceded(pair(specific_field_name(name), space0), rest_of_line)
    }

    /// creates a parser for a possibly-multi-line field with a specific name.
    ///
    /// The created parser returns the value, including any newlines and (on second lines and beyond) leading blanks,
    /// and a leading newline if the first line is blank
    pub fn named_multi_line_field<'a>(
        name: &'static str,
    ) -> impl Parser<&'a str, &'a str, nom::error::Error<&'a str>> {
        preceded(
            pair(specific_field_name(name), space0),
            recognize(pair(rest_of_line, many0(continuation_line))),
        )
    }

    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    pub struct Field<'a> {
        pub field_name: &'a str,
        pub value: &'a str,
    }
    pub fn field(input: &str) -> IResult<&str, Field<'_>> {
        map(
            separated_pair(
                field_name,
                space0,
                recognize(pair(rest_of_line, many0(continuation_line))),
            ),
            |(field_name, value)| Field { field_name, value },
        )(input)
    }
    // pub fn field_value(input: &str) -> IResult<&str, &str> {}
    #[cfg(test)]
    mod tests {
        use super::end_of_line_or_string;
        use super::field;
        use super::field_name;
        use super::line;
        use super::paragraph;
        use super::rest_of_line;
        use super::Field;

        #[test]
        fn test_field_name() {
            let (i, o) = field_name(&"asdf: ").unwrap();
            assert_eq!(o, "asdf");
            assert_eq!(i, " ");
        }

        #[test]
        fn test_field() {
            let (i, o) = field(&"asdf: jkl").unwrap();
            assert_eq!(
                o,
                Field {
                    field_name: "asdf",
                    value: "jkl"
                }
            );
            assert_eq!(i, "");
        }

        #[test]
        fn test_eol() {
            let (i, o) = end_of_line_or_string(&"\nasdf").expect("have an line ending");
            assert_eq!(o, "\n");

            let (i, o) = end_of_line_or_string(&"").expect("have an end of input");
        }

        #[test]
        fn test_line() {
            let (i, o) = line(&"asdf\njkl").expect("have a line");
            assert_eq!(o, "asdf\n");
            assert_eq!(i, "jkl");
        }

        #[test]
        fn test_rest_of_line() {
            let (i, o) = rest_of_line(&"asdf\njkl").expect("have a line");
            assert_eq!(o, "asdf\n");
            assert_eq!(i, "jkl");

            let (i, o) = rest_of_line(&"\njkl").expect("have a line");
            assert_eq!(o, "\n");
            assert_eq!(i, "jkl");

            let (i, o) = rest_of_line(&"").expect("end of string ok");
            assert!(o.is_empty());
            assert!(i.is_empty());
        }
        #[test]
        fn test_paragraph() {
            const INPUT: &str = "asdf: jkl
foo:
  bar

baz: baz
            ";

            let (i, o) = paragraph(INPUT).expect("have a paragraph");
            assert_eq!(o.len(), 3);
        }
    }
}
pub mod copyright_file {

    #[derive(Debug, Clone, PartialEq)]
    pub struct Format(pub String);

    #[derive(Debug, Clone, PartialEq)]
    pub struct UpstreamName(pub String);

    #[derive(Debug, Clone, PartialEq)]
    pub struct UpstreamContact(pub String);

    #[derive(Debug, Clone, PartialEq)]
    pub struct Source(pub String);

    #[derive(Debug, Clone, PartialEq)]
    pub struct Disclaimer(pub String);

    #[derive(Debug, Clone, PartialEq)]
    pub struct Comment(pub String);

    #[derive(Debug, Clone, PartialEq)]
    pub struct License(pub String);

    #[derive(Debug, Clone, PartialEq)]
    pub struct Copyright(pub String);

    #[derive(Debug, Clone, PartialEq)]
    pub struct Files(pub Vec<String>);

    #[derive(Debug, Clone, PartialEq)]
    pub struct HeaderParagraph {
        pub format: Format,
        pub upstream_name: Option<UpstreamName>,
        pub upstream_contact: Option<UpstreamContact>,
        pub source: Option<Source>,
        pub disclaimer: Option<Disclaimer>,
        pub comment: Option<Comment>,
        pub license: Option<License>,
        pub copyright: Option<Copyright>,
    }
    use nom::{
        branch::alt,
        bytes::complete::{tag, take_till, take_until, take_while},
        character::{
            self,
            complete::{alphanumeric1, crlf, line_ending, not_line_ending, one_of, space0, space1},
        },
        combinator::{consumed, eof, map, map_parser, not, opt, recognize, value},
        error::context,
        multi::{many0, many1},
        sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
        IResult, Parser,
    };

    use super::control_file::{self, field_pair, rest_of_line, specific_field_name, Field};

    pub fn format<'a>(input: &'a str) -> IResult<&'a str, Format> {
        map(
            preceded(pair(specific_field_name("Format"), space0), rest_of_line),
            |v| Format(v.to_string()),
        )(input)
    }

    // pub fn header_paragraph(input: &str) -> IResult<&str, &str> {}

    #[cfg(test)]
    mod tests {

        #[test]
        fn test_field_name() {}
    }
}
