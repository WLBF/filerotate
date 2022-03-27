use crate::file;
use crate::file::*;
use anyhow::Result;
use nix::sys::stat::stat;
use std::fs::{create_dir, read_dir, rename};
use std::path::PathBuf;

fn move_create(src: PathBuf, dst: PathBuf, depth: Option<i32>) -> Result<()> {
    if depth.map_or(false, |n| n <= 0) {
        return Ok(());
    }

    let f_st = stat(&src)?;

    if is_file(&f_st) {
        rename(&src, &dst)?;
        return Ok(());
    }

    if is_dir(&f_st) {
        create_dir(&dst)?;
        let entries = read_dir(&src)?;
        for res in entries {
            let entry = res?;
            let nxt_src = entry.path();
            let nxt_dst = dst.join(nxt_src.file_name().unwrap());
            move_create(nxt_src, nxt_dst, depth.map(|n| n - 1))?;
        }
    }

    Ok(())
}

fn copy_truncate(src: PathBuf, dst: PathBuf, depth: Option<i32>) -> Result<()> {
    if depth.map_or(false, |n| n <= 0) {
        return Ok(());
    }

    let f_st = stat(&src)?;

    if is_file(&f_st) {
        // do not copy if zero size file, see: https://man7.org/linux/man-pages/man2/lseek.2.html
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
            copy_truncate(nxt_src, nxt_dst, depth.map(|n| n - 1))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
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
}
