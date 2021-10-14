pub mod control_file {

    use nom::{
        branch::alt,
        bytes::complete::{tag, take_while, take_while1, take_while_m_n},
        character::{
            self,
            complete::{alphanumeric1, line_ending, not_line_ending, space0, space1},
        },
        combinator::{eof, flat_map, map, map_parser, opt, peek, recognize, value},
        error::context,
        multi::{many0, many1},
        sequence::{pair, preceded, separated_pair, terminated, tuple},
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
    fn line(input: &str) -> IResult<&str, &str> {
        recognize(pair(my_non_line_ending, end_of_line_or_string))(input)
    }

    pub fn paragraph(input: &str) -> IResult<&str, Vec<&str>> {
        many0(recognize(alt((
            pair(not_line_ending, line_ending),
            pair(take_while1(|c| c != '\n' && c != '\r'), eof),
        ))))(input)
    }

    fn rest_of_line(input: &str) -> IResult<&str, &str> {
        recognize(pair(opt(not_line_ending), end_of_line_or_string))(input)
    }
    fn continuation_line(input: &str) -> IResult<&str, &str> {
        recognize(tuple((space1, my_non_line_ending, end_of_line_or_string)))(input)
    }

    fn field_pair(input: &str) -> IResult<&str, (&str, &str)> {
        separated_pair(
            field_name,
            space0,
            recognize(pair(rest_of_line, many0(continuation_line))),
        )(input)
    }

    pub fn field_string(input: &str) -> IResult<&str, &str> {
        recognize(field_pair)(input)
    }

    /// creates a parser for a field name
    fn specific_field_name<'a>(name: &'static str) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
        value((), map_parser(field_name, tag(name)))
    }

    /// creates a parser for a single-line field with a specific name.
    ///
    /// The created parser returns the value, including the trailing line ending.
    pub fn named_single_line_field<'a>(
        name: &'static str,
    ) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
        preceded(pair(specific_field_name(name), space0), rest_of_line)
    }

    /// creates a parser for a possibly-multi-line field with a specific name.
    ///
    /// The created parser returns the value, including any newlines and (on second lines and beyond) leading blanks,
    /// and a leading newline if the first line is blank
    pub fn named_multi_line_field<'a>(
        name: &'static str,
    ) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
        preceded(
            pair(specific_field_name(name), space0),
            recognize(pair(rest_of_line, many0(continuation_line))),
        )
    }
    /// creates a parser to clean a continuation line with the given initial indent
    fn clean_continuation_lines<'a>(
        // max_indent: usize,
        indent: &'a str,
    ) -> impl Parser<&'a str, Vec<&'a str>, nom::error::Error<&'a str>> {
        let max_indent = indent.len();

        // takes some number of spaces not to exceed max_indent, and returns ()
        let take_up_to_max_indent =
            move |input: &'a str| value((), take_while_m_n(1, max_indent, |c| c == ' '))(input);

        // take exactly the indent originally passed, and returns ()
        let take_indent = value((), tag(indent));

        many1(alt((
            preceded(pair(space1, tag(".")), end_of_line_or_string),
            preceded(alt((take_indent, take_up_to_max_indent)), rest_of_line),
        )))
    }

    /// Cleans a multi-line string, assuming that the first newline is on the same line as the field name.
    pub fn clean_multiline(input: &str) -> IResult<&str, Vec<&str>> {
        map(
            pair(
                line,
                flat_map(
                    // get leading spaces on first continuation line without consuming
                    peek(space1),
                    // create a parser that trims up to that many leading spaces and apply it
                    clean_continuation_lines,
                ),
            ),
            // the continuations lines are in a vec, but the first line is not.
            // fix that.
            |(first_line, mut vec)| {
                vec.insert(0, first_line);
                vec
            },
        )(input)
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
        use nom::combinator::all_consuming;

        use crate::parser::control_file::clean_multiline;
        use crate::parser::control_file::paragraph;

        use super::end_of_line_or_string;
        use super::field;
        use super::field_name;
        use super::line;

        use super::rest_of_line;
        use super::Field;

        #[test]
        fn test_field_name() {
            let (i, o) = field_name("asdf: ").unwrap();
            assert_eq!(o, "asdf");
            assert_eq!(i, " ");
        }

        #[test]
        fn test_field() {
            let (i, o) = field("asdf: jkl").unwrap();
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
            let (_i, o) = end_of_line_or_string("\nasdf").expect("have an line ending");
            assert_eq!(o, "\n");

            let (_i, _o) = end_of_line_or_string("").expect("have an end of input");
        }

        #[test]
        fn test_line() {
            let (i, o) = line("asdf\njkl").expect("have a line");
            assert_eq!(o, "asdf\n");
            assert_eq!(i, "jkl");
        }

        #[test]
        fn test_rest_of_line() {
            let (i, o) = rest_of_line("asdf\njkl").expect("have a line");
            assert_eq!(o, "asdf\n");
            assert_eq!(i, "jkl");

            let (i, o) = rest_of_line("\njkl").expect("have a line");
            assert_eq!(o, "\n");
            assert_eq!(i, "jkl");

            let (i, o) = rest_of_line("").expect("end of string ok");
            assert!(o.is_empty());
            assert!(i.is_empty());
        }
        #[test]
        fn test_paragraph() {
            let input: &str = r#"
asdf: jkl
foo:
  bar

baz: baz
                    "#
            .trim_start();
            let (i, o) = paragraph(input).expect("have a paragraph");
            assert_eq!(o.len(), 3);
        }
        #[test]
        fn test_named_single_line() {
            use super::named_single_line_field;
            let (_i, o) = named_single_line_field("Format")(
                "Format: http://www.debian.org/doc/packaging-manuals/copyright-format/1.0/",
            )
            .expect("this is valid");
            assert_eq!(
                o,
                "http://www.debian.org/doc/packaging-manuals/copyright-format/1.0/"
            )
        }

        #[test]
        fn test_clean_continuation() {
            let (i, o) = all_consuming(clean_multiline)("0\n  a\n    .\n  b").expect("have a line");
            assert_eq!(o, vec!["0\n", "a\n", "\n", "b"]);
            assert!(i.is_empty());

            // one line is less indented but still indented
            let (i, o) = all_consuming(clean_multiline)("0\n  a\n  .\n b").expect("have a line");
            assert_eq!(o, vec!["0\n", "a\n", "\n", "b"]);
            assert!(i.is_empty());

            // One line is more indented
            let (i, o) =
                all_consuming(clean_multiline)("0\n  a\n    .\n   b").expect("have a line");
            assert_eq!(o, vec!["0\n", "a\n", "\n", " b"]);
            assert!(i.is_empty());
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
    use nom::{combinator::map, IResult};

    use super::control_file::named_single_line_field;

    pub fn format(input: &str) -> IResult<&str, Format> {
        map(named_single_line_field("Format"), |v| Format(v.to_string()))(input)
    }

    pub fn upstream_name(input: &str) -> IResult<&str, UpstreamName> {
        map(named_single_line_field("Upstream-Name"), |v| {
            UpstreamName(v.to_string())
        })(input)
    }
    // pub fn header_paragraph(input: &str) -> IResult<&str, &str> {}

    #[cfg(test)]
    mod tests {
        use crate::parser::copyright_file::Format;

        #[test]
        fn test_format() {
            use super::format;
            let (_i, o) =
                format("Format: http://www.debian.org/doc/packaging-manuals/copyright-format/1.0/")
                    .expect("this is valid");
            assert_eq!(
                o,
                Format(
                    "http://www.debian.org/doc/packaging-manuals/copyright-format/1.0/".to_string()
                )
            )
        }
    }
}
