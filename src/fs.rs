use std::fs::File;
use std::os::unix::io::RawFd;
use std::path::PathBuf;
use anyhow::Result;

const BUF_SIZE: usize = 4096;
const KIB: usize = 1024;
const MIB: usize = KIB * 1024;
const GIB: usize = MIB * 1024;

fn copy_truncate(src: PathBuf, dst: PathBuf) {
    let _src_f = File::open(src).unwrap();
    let _dst_f = File::open(dst).unwrap();
}

fn sparse_copy(_src_fd: RawFd, _dst_fd: RawFd) -> Result<()> {
    unimplemented!()
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::min;
    use std::path::Path;
    use rand::prelude::*;
    use anyhow::Result;
    use nix::fcntl::{open, OFlag};
    use nix::libc::{off_t};
    use nix::unistd::{lseek, write, close, Whence};
    use nix::sys::stat::Mode;
    use tempfile::tempdir;


    fn create_with_leading_hole(path: &Path, offset: usize, size: usize) -> Result<()> {
        let fd = open(path, OFlag::O_CREAT | OFlag::O_WRONLY, Mode::empty())?;
        lseek(fd, offset as off_t, Whence::SeekSet)?;

        let mut rng = rand::thread_rng();

        let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
        for i in (0..size).step_by(BUF_SIZE) {
            let end = min(size - i, BUF_SIZE);
            rng.fill(&mut buf[..end]);
            write(fd, &buf[..end])?;
        }

        close(fd)?;
        Ok(())
    }

    #[test]
    fn create_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("temp");
        println!("{:?}", path.as_path());
        create_with_leading_hole(&path, 10 * KIB, 8 * KIB).unwrap();
    }
}
