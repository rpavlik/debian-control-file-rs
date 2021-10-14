// Copyright 2021, Collabora, Ltd.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

use nom::{combinator::map, multi::many1, IResult};

use crate::control_file::{multi_line_field, named_single_line_field, FieldName};

pub trait ParseField: Sized {
    fn parse(input: &str) -> IResult<&str, Self>;
}
pub trait SingleLineField: FieldName {}

#[derive(Debug, Clone, PartialEq)]
pub struct Format(pub String);
impl FieldName for Format {
    const NAME: &'static str = "Format";
}
impl SingleLineField for Format {}
impl From<String> for Format {
    fn from(v: String) -> Self {
        Self(v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpstreamName(pub String);
impl FieldName for UpstreamName {
    const NAME: &'static str = "Upstream-Name";
}
impl SingleLineField for UpstreamName {}
impl From<String> for UpstreamName {
    fn from(v: String) -> Self {
        Self(v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpstreamContact(pub String);
impl FieldName for UpstreamContact {
    const NAME: &'static str = "Upstream-Contact";
}
impl SingleLineField for UpstreamContact {}
impl From<String> for UpstreamContact {
    fn from(v: String) -> Self {
        Self(v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Source(pub String);
impl FieldName for Source {
    const NAME: &'static str = "Source";
}
impl SingleLineField for Source {}
impl From<String> for Source {
    fn from(v: String) -> Self {
        Self(v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Disclaimer(pub String);
impl FieldName for Disclaimer {
    const NAME: &'static str = "Disclaimer";
}
impl ParseField for Disclaimer {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(multi_line_field::<Self>, |v| Self(v.to_owned()))(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Comment(pub String);
impl FieldName for Comment {
    const NAME: &'static str = "Comment";
}
impl ParseField for Comment {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(multi_line_field::<Self>, |v| Self(v.to_owned()))(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct License(pub String);
impl FieldName for License {
    const NAME: &'static str = "License";
}
impl ParseField for License {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(multi_line_field::<Self>, |v| Self(v.to_owned()))(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Copyright(pub Vec<String>);
impl FieldName for Copyright {
    const NAME: &'static str = "Copyright";
}

#[derive(Debug, Clone, PartialEq)]
pub struct Files(pub Vec<String>);
impl FieldName for Files {
    const NAME: &'static str = "Files";
}

impl<T: SingleLineField + From<String>> ParseField for T {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(named_single_line_field(T::NAME), |v| v.to_string().into())(input)
    }
}
fn parse_field_with_trimmed_list<T: FieldName>(input: &str) -> IResult<&str, Vec<String>> {
    map(many1(multi_line_field::<T>), |lines| {
        lines
            .into_iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    })(input)
}
impl ParseField for Copyright {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(parse_field_with_trimmed_list::<Self>, |v| Self(v))(input)
    }
}
impl ParseField for Files {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(parse_field_with_trimmed_list::<Self>, |v| Self(v))(input)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_format() {
        use super::Format;
        use super::ParseField;
        let (_i, o) = Format::parse(
            "Format: http://www.debian.org/doc/packaging-manuals/copyright-format/1.0/",
        )
        .expect("this is valid");
        assert_eq!(
            o,
            Format("http://www.debian.org/doc/packaging-manuals/copyright-format/1.0/".to_string())
        )
    }
}
