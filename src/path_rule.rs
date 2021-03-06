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

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use crate::rotate::MAX_KEEP_NUM;

pub trait PathRule {
    fn delete_paths(&self) -> &Vec<PathBuf>;
    fn rename_paths(&self) -> &Vec<PathBuf>;
    fn init_path(&self) -> Option<PathBuf>;
    fn next_path(&self, path: &Path) -> Option<PathBuf>;
}

pub struct DefaultRule {
    set: Vec<PathBuf>,
    deletes: Vec<PathBuf>,
    renames: Vec<PathBuf>,
    init_opt: Option<PathBuf>,
}

impl DefaultRule {
    pub fn new(init: PathBuf, paths: Vec<PathBuf>, keep: usize) -> DefaultRule {
        assert!(keep <= MAX_KEEP_NUM && keep > 1);
        let pos = keep - 1;
        let mut set = vec![];
        let mut delete_set = HashSet::new();
        let mut rename_set = HashSet::new();
        let init_name = init.file_name().unwrap().to_os_string();

        set.push(init.clone());

        for i in 1..pos {
            let mut name = init_name.clone();
            let mut path = init.clone();
            name.push(".");
            name.push(i.to_string());
            path.set_file_name(name);
            rename_set.insert(path.clone());
            set.push(path);
        }

        for i in pos..MAX_KEEP_NUM {
            let mut name = init_name.clone();
            let mut path = init.clone();
            name.push(".");
            name.push(i.to_string());
            path.set_file_name(name);
            delete_set.insert(path.clone());
            set.push(path)
        }

        let mut deletes = vec![];
        let mut renames = vec![];
        let mut init_opt = None;

        for p in paths.iter() {
            if delete_set.contains(p) {
                deletes.push(p.into());
            }

            if rename_set.contains(p) {
                renames.push(p.into());
            }

            if init.eq(p) {
                init_opt.replace(p.into());
            }
        }

        renames.sort_by(|a: &PathBuf, b: &PathBuf| b.cmp(a));

        DefaultRule {
            set,
            deletes,
            renames,
            init_opt,
        }
    }
}

impl PathRule for DefaultRule {
    fn delete_paths(&self) -> &Vec<PathBuf> {
        &self.deletes
    }

    fn rename_paths(&self) -> &Vec<PathBuf> {
        &self.renames
    }

    fn init_path(&self) -> Option<PathBuf> { self.init_opt.clone() }

    fn next_path(&self, path: &Path) -> Option<PathBuf> {
        let n = self.set.len();
        for i in 0..n - 1 {
            if self.set[i] == path {
                return Some(self.set[i + 1].clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn default_rule_simple_test() {
        let init = PathBuf::from("/var/lib/log");
        let paths = vec![
            PathBuf::from("/var/lib/log"),
            PathBuf::from("/var/lib/log.1"),
            PathBuf::from("/var/lib/log.2"),
            PathBuf::from("/var/lib/log.3"),
            PathBuf::from("/var/lib/log.4"),
        ];
        let rule = DefaultRule::new(init, paths, 4);
        assert_eq!(rule.renames, vec![
            PathBuf::from("/var/lib/log.2"),
            PathBuf::from("/var/lib/log.1"),
        ]);

        assert_eq!(rule.deletes, vec![
            PathBuf::from("/var/lib/log.3"),
            PathBuf::from("/var/lib/log.4"),
        ]);

        assert_eq!(rule.init_path(), Some(PathBuf::from("/var/lib/log")));
        assert_eq!(rule.next_path(&PathBuf::from("/var/lib/log")), Some(PathBuf::from("/var/lib/log.1")));
        assert_eq!(rule.next_path(&PathBuf::from("/var/lib/log.1")), Some(PathBuf::from("/var/lib/log.2")));
        assert_eq!(rule.next_path(&PathBuf::from("/var/lib/log.9")), None);
    }

    #[test]
    fn default_rule_missing_test() {
        let init = PathBuf::from("/var/lib/log");
        let paths = vec![
            PathBuf::from("/var/lib/log.1"),
            PathBuf::from("/var/lib/log.2"),
            PathBuf::from("/var/lib/log.5"),
            PathBuf::from("/var/lib/log.6"),
        ];
        let rule = DefaultRule::new(init, paths, 3);
        assert_eq!(rule.renames, vec![
            PathBuf::from("/var/lib/log.1"),
        ]);

        assert_eq!(rule.deletes, vec![
            PathBuf::from("/var/lib/log.2"),
            PathBuf::from("/var/lib/log.5"),
            PathBuf::from("/var/lib/log.6"),
        ]);

        assert_eq!(rule.init_path(), None);
        assert_eq!(rule.next_path(&PathBuf::from("/var/lib/log.1")), Some(PathBuf::from("/var/lib/log.2")));
        assert_eq!(rule.next_path(&PathBuf::from("/var/lib/log.3")), Some(PathBuf::from("/var/lib/log.4")));
    }
}
