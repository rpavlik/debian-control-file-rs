// Copyright 2021, Collabora, Ltd.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

pub mod fields;

use self::fields::{
    Comment, Copyright, Disclaimer, Format, License, Source, UpstreamContact, UpstreamName,
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
