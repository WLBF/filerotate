use std::ffi::OsStr;
use std::fmt;
use crate::file;
use crate::file::*;
use crate::path_rule::*;
use crate::regex::Regex;
use anyhow::{Result};
use nix::sys::stat::{FileStat, stat};
use std::fs::{create_dir, read_dir, rename, remove_file, File, remove_dir_all};
use std::path::{PathBuf};
use std::process::Command;
use serde::{Serialize, Deserialize, Deserializer};

#[derive(Deserialize, Debug)]
pub enum Mode {
    MoveCreate,
    CopyTruncate,
}

#[derive(Deserialize, Debug)]
pub struct Rotate {
    path: PathBuf,
    keep: usize,
    #[serde(rename = "depth")]
    depth_opt: Option<i32>,
    #[serde(rename = "size")]
    sz_opt: Option<i64>,
    #[serde(rename = "regex")]
    re_opt: Option<Regex>,
    #[serde(rename = "precmd")]
    pre_opt: Option<Vec<String>>,
    #[serde(rename = "postcmd")]
    post_opt: Option<Vec<String>>,
    mode: Mode,
}

impl Rotate {
    pub fn rotate(&self) -> Result<()> {
        let f_st = stat(&self.path)?;

        if is_file(&f_st) {
            // check if size hit threshold
            if !size_check(self.sz_opt, f_st) {
                return Ok(());
            }

            // check if name match regex
            if !regex_check(self.re_opt.as_ref(), &self.path) {
                return Ok(());
            }
        }

        let parent = self.path.parent().unwrap();
        let entries = read_dir(parent)?;
        let mut paths = vec![];
        for res in entries {
            paths.push(res?.path());
        }

        let rule = DefaultRule::new(self.path.clone(), paths, self.keep);

        for p in rule.delete_paths().iter() {
            if p.is_file() {
                remove_file(p)?;
            } else if p.is_dir() {
                remove_dir_all(p)?;
            }
        }

        for p in rule.rename_paths().iter() {
            rename(p, rule.next_path(p).unwrap());
        }

        if let Some(p) = rule.init_path() {
            if let Some(cmd) = &self.pre_opt {
                Command::new(&cmd[0])
                    .args(&cmd[1..])
                    .output()?;
            }

            match self.mode {
                Mode::MoveCreate => move_create(p.clone(), rule.next_path(&p).unwrap(), self.depth_opt, self.sz_opt, self.re_opt.as_ref())?,
                Mode::CopyTruncate => copy_truncate(p.clone(), rule.next_path(&p).unwrap(), self.depth_opt, self.sz_opt, self.re_opt.as_ref())?,
            }

            if let Some(cmd) = &self.post_opt {
                Command::new(&cmd[0])
                    .args(&cmd[1..])
                    .output()?;
            }
        }

        Ok(())
    }
}

fn size_check(sz_opt: Option<i64>, f_st: FileStat) -> bool {
    match sz_opt {
        Some(sz) => f_st.st_blocks * 512 > sz,
        None => true
    }
}

fn regex_check(re_opt: Option<&Regex>, path: &PathBuf) -> bool {
    match (re_opt, path.file_name().unwrap().to_str()) {
        (Some(re), Some(name)) => re.is_match(name),
        (None, _) => true,
        _ => false,
    }
}

fn move_create(src: PathBuf, dst: PathBuf, depth_opt: Option<i32>, sz_opt: Option<i64>, re_opt: Option<&Regex>) -> Result<()> {
    if depth_opt.map_or(false, |n| n <= 0) {
        return Ok(());
    }

    let f_st = stat(&src)?;

    if is_file(&f_st) {
        // check if size hit threshold
        if !size_check(sz_opt, f_st) {
            return Ok(());
        }

        // check if name match regex
        if !regex_check(re_opt, &src) {
            return Ok(());
        }

        rename(&src, &dst)?;
        File::create(&src)?;
        return Ok(());
    }

    if is_dir(&f_st) {
        create_dir(&dst)?;
        let entries = read_dir(&src)?;
        for res in entries {
            let entry = res?;
            let nxt_src = entry.path();
            let nxt_dst = dst.join(nxt_src.file_name().unwrap());
            move_create(nxt_src, nxt_dst, depth_opt.map(|n| n - 1), sz_opt, re_opt)?;
        }
    }

    Ok(())
}

fn copy_truncate(src: PathBuf, dst: PathBuf, depth_opt: Option<i32>, sz_opt: Option<i64>, re_opt: Option<&Regex>) -> Result<()> {
    if depth_opt.map_or(false, |n| n <= 0) {
        return Ok(());
    }

    let f_st = stat(&src)?;

    if is_file(&f_st) {
        // check if size hit threshold
        if !size_check(sz_opt, f_st) {
            return Ok(());
        }

        // check if name match regex
        if !regex_check(re_opt, &src) {
            return Ok(());
        }

        // do not copy zero size file, see: https://man7.org/linux/man-pages/man2/lseek.2.html
        if stat_size(&f_st) > 0 {
            file::copy_truncate(&src, &dst)?;
        }
        return Ok(());
    }

    if is_dir(&f_st) {
        create_dir(&dst)?;
        let entries = read_dir(&src)?;
        for res in entries {
            let entry = res?;
            let nxt_src = entry.path();
            let nxt_dst = dst.join(nxt_src.file_name().unwrap());
            copy_truncate(nxt_src, nxt_dst, depth_opt.map(|n| n - 1), sz_opt, re_opt)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use super::*;
    use std::fs::DirEntry;
    use std::fs::{metadata};
    use tempfile::tempdir;

    #[derive(Eq, PartialEq, Debug)]
    enum Node {
        File { name: String },
        Dir { name: String, children: Vec<Node> },
    }

    fn build_tree(dir: PathBuf, root: &Node) {
        match root {
            Node::File { name } => {
                let path = dir.join(&name);
                create_with_leading_hole(&path, 4096, 4096).unwrap();
            }
            Node::Dir { name, children } => {
                let path = dir.join(&name);
                create_dir(&path).unwrap();
                for node in children.iter() {
                    build_tree(path.clone(), node)
                }
            }
        }
    }

    fn inspect_tree(root: &Node, path: PathBuf) -> bool {
        let meta = metadata(&path).unwrap();
        match (meta.is_file(), root) {
            (true, Node::File { name }) => path.file_name().unwrap().to_str().unwrap() == name,
            (false, Node::Dir { name, children }) => {
                if path.file_name().unwrap().to_str().unwrap() != name {
                    return false;
                }

                let entries: Vec<DirEntry> =
                    read_dir(&path).unwrap().map(|res| res.unwrap()).collect();
                if entries.len() != children.len() {
                    return false;
                }

                let mut cnt = 0;
                for node in children.iter() {
                    for entry in entries.iter() {
                        if inspect_tree(node, entry.path()) {
                            cnt += 1;
                        }
                    }
                }

                cnt == children.len()
            }
            _ => false,
        }
    }

    fn gen_tree(root: &str) -> Node {
        Node::Dir {
            name: root.to_string(),
            children: vec![
                Node::File {
                    name: "file0".to_string(),
                },
                Node::File {
                    name: "file1".to_string(),
                },
                Node::Dir {
                    name: "dir1".to_string(),
                    children: vec![Node::File {
                        name: "file2".to_string(),
                    }],
                },
            ],
        }
    }

    #[test]
    fn build_and_inspect_tree_test() {
        let path = tempdir().unwrap().into_path();
        // let path = PathBuf::new();
        let tree = gen_tree("dir0");

        build_tree(path.clone(), &tree);
        assert!(inspect_tree(&tree, path.join("dir0")));
    }

    #[test]
    fn move_create_simple_test() {
        let path = tempdir().unwrap().into_path();
        // let path = PathBuf::new();

        let tree0 = gen_tree("dir0");
        let tree1 = gen_tree("dir0.1");
        let path0 = path.join("dir0");
        let path1 = path.join("dir0.1");

        build_tree(path, &tree0);
        move_create(path0, path1.clone(), None).unwrap();

        assert!(inspect_tree(&tree1, path1));
    }

    #[test]
    fn move_create_recursive_test() {
        let path = tempdir().unwrap().into_path();
        // let path = PathBuf::new();

        let tree0 = gen_tree("dir0");
        let tree1 = Node::Dir {
            name: "dir0.1".to_string(),
            children: vec![
                Node::File {
                    name: "file0".to_string(),
                },
                Node::File {
                    name: "file1".to_string(),
                },
                Node::Dir {
                    name: "dir1".to_string(),
                    children: vec![],
                },
            ],
        };

        let path0 = path.join("dir0");
        let path1 = path.join("dir0.1");

        build_tree(path, &tree0);
        move_create(path0, path1.clone(), Some(2)).unwrap();

        assert!(inspect_tree(&tree1, path1));
    }

    #[test]
    fn copy_truncate_simple_test() {
        let path = tempdir().unwrap().into_path();
        // let path = PathBuf::new();

        let tree0 = gen_tree("dir0");
        let tree1 = gen_tree("dir0.1");
        let path0 = path.join("dir0");
        let path1 = path.join("dir0.1");

        build_tree(path, &tree0);
        copy_truncate(path0, path1.clone(), None).unwrap();

        assert!(inspect_tree(&tree1, path1));
    }

    #[test]
    fn rotate_simple_test() {
        let path = tempdir().unwrap().into_path();
        // let path = env::current_dir().unwrap();

        let tree0 = gen_tree("dir0");
        let tree1 = gen_tree("dir0.1");
        let tree2 = gen_tree("dir0.2");
        let path0 = path.join("dir0");
        let path1 = path.join("dir0.1");
        let path2 = path.join("dir0.2");
        let path3 = path.join("dir0.3");

        build_tree(path, &tree0);

        rotate(path0.clone(), 2, None).unwrap();
        assert!(inspect_tree(&tree0, path0.clone()));
        assert!(inspect_tree(&tree1, path1.clone()));

        rotate(path0.clone(), 2, None).unwrap();
        assert!(inspect_tree(&tree0, path0.clone()));
        assert!(inspect_tree(&tree1, path1.clone()));
        assert!(inspect_tree(&tree2, path2.clone()));

        rotate(path0.clone(), 2, None).unwrap();
        assert!(inspect_tree(&tree0, path0.clone()));
        assert!(inspect_tree(&tree1, path1.clone()));
        assert!(inspect_tree(&tree2, path2.clone()));
        assert!(!path3.exists());

        rotate(path0.clone(), 2, None).unwrap();
        assert!(inspect_tree(&tree0, path0));
        assert!(inspect_tree(&tree1, path1));
        assert!(inspect_tree(&tree2, path2));
        assert!(!path3.exists());
    }

    #[test]
    fn rotate_missing_test() {
        // let path = tempdir().unwrap().into_path();
        let path = env::current_dir().unwrap();

        let tree1 = gen_tree("dir0.1");
        let tree2 = gen_tree("dir0.2");
        let path0 = path.join("dir0");
        let path1 = path.join("dir0.1");
        let path2 = path.join("dir0.2");
        let path3 = path.join("dir0.3");

        build_tree(path.clone(), &tree1);
        build_tree(path, &tree2);

        rotate(path0.clone(), 2, None).unwrap();
        assert!(inspect_tree(&tree2, path2.clone()));
        assert!(!path0.exists());
        assert!(!path1.exists());
        assert!(!path3.exists());

        rotate(path0.clone(), 2, None);
        assert!(!path0.exists());
        assert!(!path1.exists());
        assert!(!path2.exists());
        assert!(!path3.exists());
    }
}
