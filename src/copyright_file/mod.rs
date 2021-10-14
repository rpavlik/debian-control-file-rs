// Copyright 2021, Collabora, Ltd.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

pub mod fields;

use nom::{
    branch::{alt, permutation},
    character::complete::{line_ending, space0, space1},
    combinator::{map, map_parser, opt},
    multi::many0,
    sequence::{delimited, pair, preceded},
    IResult,
};

use crate::control_file::{cleaned_multiline, named_multi_line_field};

use self::fields::{
    Comment, Copyright, Disclaimer, Files, Format, License, ParseField, Source, UpstreamContact,
    UpstreamName,
};

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

pub fn header_paragraph(input: &str) -> IResult<&str, HeaderParagraph> {
    map(
        permutation((
            Format::parse,
            opt(UpstreamName::parse),
            opt(UpstreamContact::parse),
            opt(Source::parse),
            opt(Disclaimer::parse),
            opt(Comment::parse),
            opt(License::parse),
            opt(Copyright::parse),
        )),
        |(
            format,
            upstream_name,
            upstream_contact,
            source,
            disclaimer,
            comment,
            license,
            copyright,
        )| HeaderParagraph {
            format,
            upstream_name,
            upstream_contact,
            source,
            disclaimer,
            comment,
            license,
            copyright,
        },
    )(input)
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilesParagraph {
    pub files: Files,
    pub copyright: Copyright,
    pub license: License,
    pub comment: Option<Comment>,
}

pub fn files_paragraph(input: &str) -> IResult<&str, FilesParagraph> {
    map(
        permutation((
            Files::parse,
            Copyright::parse,
            License::parse,
            opt(Comment::parse),
        )),
        |(files, copyright, license, comment)| FilesParagraph {
            files,
            copyright,
            license,
            comment,
        },
    )(input)
}

#[derive(Debug, Clone, PartialEq)]
pub struct LicenseDetailParagraph {
    pub name: String,
    pub text: String,
}

pub fn license_detail_paragraph(input: &str) -> IResult<&str, LicenseDetailParagraph> {
    map(
        map_parser(named_multi_line_field("License"), cleaned_multiline),
        |lines: Vec<&str>| {
            let mut iter = lines.into_iter();
            let name = iter.next().unwrap().to_string();
            let text = iter
                .map(|line| line.trim_end())
                .collect::<Vec<_>>()
                .join("\n");
            LicenseDetailParagraph { name, text }
        },
    )(input)
}

pub enum BodyParagraph {
    Files(FilesParagraph),
    LicenseDetail(LicenseDetailParagraph),
}

pub fn body_paragraph(input: &str) -> IResult<&str, BodyParagraph> {
    preceded(
        many0(pair(space0, line_ending)),
        alt((
            map(files_paragraph, |v| BodyParagraph::Files(v)),
            map(license_detail_paragraph, |v| {
                BodyParagraph::LicenseDetail(v)
            }),
        )),
    )(input)
}

pub struct CopyrightFile {
    pub header_paragraph: HeaderParagraph,
    pub body_paragraphs: Vec<BodyParagraph>,
}

pub fn copyright_file(input: &str) -> IResult<&str, CopyrightFile> {
    map(
        delimited(
            many0(pair(space0, line_ending)),
            pair(header_paragraph, many0(body_paragraph)),
            space0,
        ),
        |(header_paragraph, body_paragraphs)| CopyrightFile {
            header_paragraph,
            body_paragraphs,
        },
    )(input)
}
#[cfg(test)]
mod tests {

    #[test]
    fn test_format() {
        use super::copyright_file;
        let (_i, o) = copyright_file(
            r#"
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/

Files: CONTRIBUTING.md
Copyright: 2018-2019 Collabora, Ltd.
    and License for this CONTRIBUTING.md file
License: CC-BY-4.0

Files: README.md
Copyright: 2018-2020, Collabora, Ltd.
License: CC-BY-4.0

Files: doc/CHANGELOG.md
Copyright: 2020 Collabora, Ltd. and the Monado contributors
License: CC0-1.0

Files: doc/changes/*
Copyright: 2020 Collabora, Ltd. and the Proclamation contributors
License: CC0-1.0

Files: doc/changes/auxiliary/*
    doc/changes/big/*
    doc/changes/compositor/*
    doc/changes/drivers/*
    doc/changes/ipc/*
    doc/changes/misc_features/*
    doc/changes/misc_fixes/*
    doc/changes/state_trackers/*
    doc/changes/xrt/*
Copyright: 2020-2021, Collabora, Ltd. and the Monado contributors
License: CC0-1.0

Files: doc/conventions.md
    doc/frame-timing.md
    doc/howto-release.md
    doc/implementing-extensions.md
    doc/ipc.md
    doc/tracing.md
Copyright: 2021, Collabora, Ltd. and the Monado contributors
License: BSL-1.0
            "#,
        )
        .expect("this is valid");
    }
}
