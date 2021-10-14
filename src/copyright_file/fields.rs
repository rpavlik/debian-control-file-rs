// Copyright 2021, Collabora, Ltd.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

use nom::{combinator::map, IResult};

use crate::control_file::{named_single_line_field, FieldName};

pub trait ParseField: Sized {
    fn parse(input: &str) -> IResult<&str, Self>;
}
trait SingleLineField: FieldName {}

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

#[derive(Debug, Clone, PartialEq)]
pub struct Comment(pub String);
impl FieldName for Comment {
    const NAME: &'static str = "Comment";
}

#[derive(Debug, Clone, PartialEq)]
pub struct License(pub String);
impl FieldName for License {
    const NAME: &'static str = "License";
}

#[derive(Debug, Clone, PartialEq)]
pub struct Copyright(pub String);
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
