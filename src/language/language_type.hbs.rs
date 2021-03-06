// Copyright (c) 2015 Aaron Power
// Use of this source code is governed by the APACHE2.0/MIT licence that can be
// found in the LICENCE-{APACHE/MIT} file.

use std::borrow::Cow;
use std::fmt;
use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::BTreeMap;

use utils::fs;
use self::LanguageType::*;
use Languages;
use Language;


#[cfg_attr(feature = "io", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LanguageType {
    {{~#each languages}}
        {{~@key}},
    {{/each}}
}

impl LanguageType {
    /// Returns the display name of a language.
    ///
    /// ```
    /// # use tokei::*;
    /// let bash = LanguageType::Bash;
    ///
    /// assert_eq!(bash.name(), "BASH");
    /// ```
    pub fn name(&self) -> &'static str {
        match *self {
            {{~#each languages}}
                {{@key}} =>
                {{#if this.name}}
                    "{{~name}}"
                {{else}}
                    "{{~@key}}"
                {{~/if}},
            {{~/each}}
        }
    }

    /// Get language from it's file extension.
    ///
    /// ```no_run
    /// # use tokei::*;
    /// let rust = LanguageType::from_extension("./main.rs");
    ///
    /// assert_eq!(rust, Some(LanguageType::Rust));
    /// ```
    pub fn from_extension<P: AsRef<Path>>(entry: P) -> Option<Self> {
        if let Some(extension) = fs::get_extension(entry) {
            match &*extension {
                {{~#each languages}}
                    {{~#each this.extensions}}
                        "{{~this}}" {{~#unless @last}} | {{~/unless}}
                    {{~/each}}
                        => Some({{~@key}}),
                {{~/each}}
                extension => {
                    warn!("Unknown extension: {}", extension);
                    None
                },
            }
        } else {
            None
        }
    }
}

impl Languages {
    #[inline]
    pub fn generate_languages() -> BTreeMap<LanguageType, Language> {
        btreemap! {
            {{~#each languages}}
                {{~@key}} =>
                {{~#if this.base}}
                    Language::new_{{this.base}}()
                {{else}}
                    {{~#if this.single}}
                        {{~#if this.multi}}
                            Language::new(
                                vec![
                                {{~#each this.single}}
                                    "{{this}}",
                                {{~/each}}
                                ],
                                vec![
                                {{~#each this.multi}}
                                    (
                                    {{~#each this}}
                                        "{{this}}",
                                    {{~/each}}
                                    ),
                                {{~/each}}
                                ]
                            )
                        {{else}}
                            Language::new_single(vec![
                                {{~#each this.single}}
                                    "{{~this}}",
                                {{~/each}}
                            ])
                        {{~/if}}
                    {{else}}
                        Language::new_multi(vec![
                            {{~#each this.multi}}
                                (
                                {{~#each this}}
                                    "{{~this}}",
                                {{~/each}}
                                ),
                            {{~/each}}
                        ])
                    {{~/if}}
                {{~/if}}
                {{~#if this.nested}}
                    .nested()
                {{~/if}}
                {{~#if this.nested_comments}}
                    .nested_comments(vec![
                        {{~#each this.nested_comments}}
                            (
                            {{~#each this}}
                                "{{this}}",
                            {{~/each}}
                            ),
                        {{~/each}}
                    ])
                {{~/if}}
                {{~#if this.quotes}}
                    .set_quotes(vec![
                        {{~#each this.quotes}}
                            (
                            {{~#each this}}
                                "{{this}}",
                            {{~/each}}
                            ),
                        {{~/each}}
                    ])
                {{~/if}},
            {{~/each}}
        }
    }
}

impl From<String> for LanguageType {
    fn from(from: String) -> Self {
        LanguageType::from(&*from)
    }
}

impl<'a> From<&'a str> for LanguageType {
    fn from(from: &str) -> Self {
        match &*from {
            {{~#each languages}}
                {{~#if this.name}}
                    "{{~this.name}}"
                {{else}}
                    "{{~@key}}"
                {{~/if}}
                    => {{~@key}},
            {{~/each}}
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for LanguageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}


impl<'a> From<LanguageType> for Cow<'a, LanguageType> {
    fn from(from: LanguageType) -> Self {
        Cow::Owned(from)
    }
}

impl<'a> From<&'a LanguageType> for Cow<'a, LanguageType> {
    fn from(from: &'a LanguageType) -> Self {
        Cow::Borrowed(from)
    }
}


/// This is for getting the file type from the first line of a file
pub fn get_filetype_from_shebang<P: AsRef<Path>>(file: P) -> Option<&'static str> {
    let file = match File::open(file) {
        Ok(file) => file,
        _ => return None,
    };
    let mut buf = BufReader::new(file);
    let mut line = String::new();
    let _ = buf.read_line(&mut line);

    let mut words = line.split_whitespace();
    match words.next() {
        Some("#!/bin/sh") => Some("sh"),
        Some("#!/bin/csh") => Some("csh"),
        Some("#!/usr/bin/perl") => Some("pl"),
        Some("#!/usr/bin/env") => {
            if let Some(word) = words.next() {
                match word {
                    {{~#each languages}}
                        {{~#if this.env}}
                            {{~#each this.env}}
                                "{{~this}}"
                                {{~#unless @last}}
                                    |
                                {{~/unless}}
                            {{~/each}}
                                => Some("{{this.extensions[0]}}"),
                        {{~/if}}
                    {{~/each}}
                    env => {
                        warn!("Unknown environment: {:?}", env);
                        None
                    }
                }
            } else {
                None
            }
        }
        _ => None,
    }
}
