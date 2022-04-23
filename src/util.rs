#![allow(dead_code)]

use anyhow::Result;
use nix::libc::off_t;
use nix::libc::{S_IFDIR, S_IFMT, S_IFREG};
use nix::sys::sendfile::sendfile;
use nix::sys::stat::{FileStat};
use nix::unistd::{ftruncate, lseek, write, Whence};
use rand::prelude::*;

use sha2::{Digest, Sha256};
use std::cmp::min;
use std::fs::File;
use std::io;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::path::Path;

#[inline(always)]
pub fn is_file(f_st: &FileStat) -> bool {
    f_st.st_mode & S_IFMT == S_IFREG
}

#[inline(always)]
pub fn is_dir(f_st: &FileStat) -> bool {
    f_st.st_mode & S_IFMT == S_IFDIR
}

#[inline(always)]
pub fn stat_size(f_st: &FileStat) -> usize {
    f_st.st_blocks as usize * 512
}

pub fn copy_truncate(src: &Path, dst: &Path) -> Result<()> {
    let src_f = File::options().read(true).write(true).open(src)?;
    let dst_f = File::create(dst)?;

    sparse_copy(src_f.as_raw_fd(), dst_f.as_raw_fd())?;
    dst_f.sync_all()?;

    ftruncate(src_f.as_raw_fd(), 0)?;

    Ok(())
}

fn sparse_copy(src_fd: RawFd, dst_fd: RawFd) -> Result<usize> {
    let offset = lseek(src_fd, 0, Whence::SeekData)?;
    lseek(dst_fd, offset, Whence::SeekSet)?;
    let mut n = 1;
    let mut sz = 0;
    while n > 0 {
        n = sendfile(dst_fd, src_fd, None, 4096 * 256)?;
        sz += n;
    }
    Ok(sz)
}

pub fn create_with_leading_hole(path: &Path, hole_size: usize, data_size: usize) -> Result<File> {
    let file = File::create(path)?;
    let fd = file.as_raw_fd();
    lseek(fd, hole_size as off_t, Whence::SeekSet)?;

    let mut rng = rand::thread_rng();
    const BUF_SZ: usize = 4096;
    let mut buf: [u8; BUF_SZ] = [0; BUF_SZ];
    for i in (0..data_size).step_by(BUF_SZ) {
        let end = min(data_size - i, BUF_SZ);
        rng.fill(&mut buf[..end]);
        write(fd, &buf[..end])?;
    }

    Ok(file)
}

fn file_digest(path: &Path) -> Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut sha256 = Sha256::new();
    io::copy(&mut file, &mut sha256)?;
    Ok(sha256.finalize().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nix::sys::stat::stat;
    use tempfile::tempdir;

    const KIB: usize = 1024;

    fn storage_size(path: &Path) -> Result<usize> {
        let file_stat = stat(path)?;
        Ok(stat_size(&file_stat))
    }

    #[test]
    fn storage_size_test() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a");
        create_with_leading_hole(&path, 16 * KIB, 8 * KIB).unwrap();
        let sz = storage_size(&path).unwrap();
        assert_eq!(sz, 8 * KIB);
    }

    #[test]
    fn sparse_copy_test() {
        let dir = tempdir().unwrap();
        let path_a = dir.path().join("a");
        let path_b = dir.path().join("b");
        create_with_leading_hole(&path_a, 16 * KIB, 8 * KIB).unwrap();
        let file_a = File::open(&path_a).unwrap();
        let file_b = File::create(&path_b).unwrap();
        sparse_copy(file_a.as_raw_fd(), file_b.as_raw_fd()).unwrap();
        let size_a = storage_size(&path_a).unwrap();
        let size_b = storage_size(&path_b).unwrap();
        let digest_a = file_digest(&path_a).unwrap();
        let digest_b = file_digest(&path_b).unwrap();
        assert_eq!(size_a, 8 * KIB);
        assert_eq!(size_b, 8 * KIB);
        assert_eq!(digest_a, digest_b);
    }

    #[test]
    fn copy_truncate_test() {
        let dir = tempdir().unwrap();
        let path_a = dir.path().join("a");
        let path_b = dir.path().join("b");
        create_with_leading_hole(&path_a, 16 * KIB, 8 * KIB).unwrap();
        let digest_a = file_digest(&path_a).unwrap();
        copy_truncate(&path_a, &path_b).unwrap();
        let size_a = storage_size(&path_a).unwrap();
        let size_b = storage_size(&path_b).unwrap();
        let digest_b = file_digest(&path_b).unwrap();
        assert_eq!(size_a, 0);
        assert_eq!(size_b, 8 * KIB);
        assert_eq!(digest_a, digest_b);
    }
}
