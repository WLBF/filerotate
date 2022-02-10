use std::os::unix::io::RawFd;
use std::path::PathBuf;
use std::path::Path;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use anyhow::Result;
use nix::unistd::{lseek, ftruncate, Whence};
use nix::sys::stat::stat;
use nix::sys::sendfile::sendfile;

const KIB: usize = 1024;
const MIB: usize = KIB * 1024;
const GIB: usize = MIB * 1024;

fn copy_truncate(src: &Path, dst: &Path) -> Result<()> {
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

fn storage_size(path: &Path) -> Result<usize> {
    let file_stat = stat(path)?;
    Ok(file_stat.st_blocks as usize * 512)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::min;
    use std::io;
    use rand::prelude::*;
    use anyhow::Result;
    use nix::libc::{off_t};
    use nix::unistd::{lseek, write, Whence};
    use tempfile::tempdir;
    use sha2::{Sha256, Digest};
    use sha2::digest::Output;

    fn create_with_leading_hole(path: &Path, offset: usize, size: usize) -> Result<File> {
        let file = File::create(path)?;
        let fd = file.as_raw_fd();
        lseek(fd, offset as off_t, Whence::SeekSet)?;

        let mut rng = rand::thread_rng();
        const BUF_SZ: usize = 4096;
        let mut buf: [u8; BUF_SZ] = [0; BUF_SZ];
        for i in (0..size).step_by(BUF_SZ) {
            let end = min(size - i, BUF_SZ);
            rng.fill(&mut buf[..end]);
            write(fd, &buf[..end])?;
        }

        Ok(file)
    }

    fn file_digest(path: &Path) -> Result<Output<Sha256>> {
        let mut file = File::open(path)?;
        let mut sha256 = Sha256::new();
        io::copy(&mut file, &mut sha256)?;
        Ok(sha256.finalize())
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
