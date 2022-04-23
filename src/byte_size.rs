//! Copyright 2021 Liu BoFan
//!
//! Licensed under the Apache License, Version 2.0 (the "License");
//! you may not use this file except in compliance with the License.
//! You may obtain a copy of the License at
//!
//!     http://www.apache.org/licenses/LICENSE-2.0
//!
//! Unless required by applicable law or agreed to in writing, software
//! distributed under the License is distributed on an "AS IS" BASIS,
//! WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//! See the License for the specific language governing permissions and
//! limitations under the License.

use anyhow::Result;
use std::fmt;
use std::str::FromStr;

const KIB: usize = 1024;
const MIB: usize = KIB * 1024;
const GIB: usize = MIB * 1024;

#[derive(Clone, Debug)]
pub struct ByteSize {
    pub bytes: usize,
    pub raw: String,
}

impl ByteSize {
    pub fn new(bytes: usize) -> Self {
        let mut raw = String::new();
        if bytes >= GIB {
            raw.push_str(&format!("{}GiB", bytes / GIB));
        } else if bytes >= MIB {
            raw.push_str(&format!("{}MiB", bytes / MIB));
        } else if bytes >= KIB {
            raw.push_str(&format!("{}KiB", bytes / KIB));
        } else {
            raw.push_str(&format!("{}B", bytes));
        }
        ByteSize { bytes, raw }
    }
}

impl FromStr for ByteSize {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let bytes ;
        let mut num = String::new();
        let mut unit = String::new();

        let mut i = 0;
        while i < s.len() && s.chars().nth(i).unwrap().is_digit(10) {
            num.push(s.chars().nth(i).unwrap());
            i += 1;
        }
        bytes = num.parse::<usize>()?;

        while i < s.len() {
            unit.push(s.chars().nth(i).unwrap());
            i += 1;
        }

        match unit.to_ascii_lowercase().as_str() {
            "" => Ok(ByteSize::new(bytes)),
            "b" => Ok(ByteSize::new(bytes)),
            "k" => Ok(ByteSize::new(bytes * KIB)),
            "kb" => Ok(ByteSize::new(bytes * KIB)),
            "kib" => Ok(ByteSize::new(bytes * KIB)),
            "m" => Ok(ByteSize::new(bytes * MIB)),
            "mb" => Ok(ByteSize::new(bytes * MIB)),
            "mib" => Ok(ByteSize::new(bytes * MIB)),
            "g" => Ok(ByteSize::new(bytes * GIB)),
            "gb" => Ok(ByteSize::new(bytes * GIB)),
            "gib" => Ok(ByteSize::new(bytes * GIB)),
            _ => Err(anyhow::anyhow!("Invalid unit")),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ByteSize {
    fn deserialize<D>(de: D) -> Result<ByteSize, D::Error>
        where D: serde::Deserializer<'de>
    {
        use serde::de::{Error, Visitor};

        struct RegexVisitor;

        impl<'de> Visitor<'de> for RegexVisitor {
            type Value = ByteSize;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a bytes size string")
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<ByteSize, E> {
                ByteSize::from_str(v).map_err(|err| {
                    E::custom(err.to_string())
                })
            }
        }

        de.deserialize_str(RegexVisitor)
    }
}
