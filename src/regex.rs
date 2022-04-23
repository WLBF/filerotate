use std::ffi::OsStr;
use std::fmt;
use crate::util;
use crate::util::*;
use crate::path_rule::*;
use anyhow::{Result};
use nix::sys::stat::{FileStat, stat};
use std::fs::{create_dir, read_dir, rename, remove_file, File, remove_dir_all};
use std::path::{PathBuf};
use serde::{Serialize, Deserialize, Deserializer};

#[derive(Clone, Debug)]
pub struct Regex(regex::Regex);
impl Regex {
    pub fn new(pattern: &str) -> Result<Self> {
        Ok(Regex(regex::Regex::new(pattern)?))
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.0.is_match(text)
    }
}

impl<'de> serde::Deserialize<'de> for Regex {
    fn deserialize<D>(de: D) -> Result<Regex, D::Error>
        where D: serde::Deserializer<'de>
    {
        use serde::de::{Error, Visitor};

        struct RegexVisitor;

        impl<'de> Visitor<'de> for RegexVisitor {
            type Value = Regex;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a regular expression pattern")
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Regex, E> {
                regex::Regex::new(v).map(Regex).map_err(|err| {
                    E::custom(err.to_string())
                })
            }
        }

        de.deserialize_str(RegexVisitor)
    }
}
