use std::ffi::{OsStr, OsString};
use std::time::{Duration, SystemTime};

use futures_util::stream;

use async_trait::async_trait;
use fuse3::prelude::*;
use fuse3::Session;

use clap::{App, Arg};

const CONTENT: &str = "hello world\n";

const PARENT_INODE: u64 = 1;
const FILE_INODE: u64 = 2;
const FILE_NAME: &str = "hello-world.txt";
const PARENT_MODE: u16 = 0o755;
const FILE_MODE: u16 = 0o644;
const TTL: Duration = Duration::from_secs(1);
const STATFS: ReplyStatFs = ReplyStatFs {
    blocks: 1,
    bfree: 0,
    bavail: 0,
    files: 1,
    ffree: 0,
    bsize: 4096,
    namelen: u32::max_value(),
    frsize: 0,
};

struct HelloWorld;

#[async_trait]
impl Filesystem for HelloWorld {
    async fn init(&self, _req: Request) -> Result<()> {
        Ok(())
    }

    async fn destroy(&self, _req: Request) {}

    async fn lookup(&self, _req: Request, parent: u64, name: &OsStr) -> Result<ReplyEntry> {
        if parent != PARENT_INODE {
            return Err(libc::ENOENT.into());
        }

        if name != OsStr::new(FILE_NAME) {
            return Err(libc::ENOENT.into());
        }

        Ok(ReplyEntry {
            ttl: TTL,
            attr: FileAttr {
                ino: FILE_INODE,
                generation: 0,
                size: CONTENT.len() as u64,
                blocks: 0,
                atime: SystemTime::now(),
                mtime: SystemTime::now(),
                ctime: SystemTime::now(),
                kind: FileType::RegularFile,
                perm: FILE_MODE,
                nlink: 0,
                uid: 0,
                gid: 0,
                rdev: 0,
                blksize: 0,
            },
            generation: 0,
        })
    }

    async fn getattr(
        &self,
        _req: Request,
        inode: u64,
        _fh: Option<u64>,
        _flags: u32,
    ) -> Result<ReplyAttr> {
        if inode == PARENT_INODE {
            Ok(ReplyAttr {
                ttl: TTL,
                attr: FileAttr {
                    ino: PARENT_INODE,
                    generation: 0,
                    size: 0,
                    blocks: 0,
                    atime: SystemTime::now(),
                    mtime: SystemTime::now(),
                    ctime: SystemTime::now(),
                    kind: FileType::Directory,
                    perm: PARENT_MODE,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 0,
                },
            })
        } else if inode == FILE_INODE {
            Ok(ReplyAttr {
                ttl: TTL,
                attr: FileAttr {
                    ino: FILE_INODE,
                    generation: 0,
                    size: CONTENT.len() as _,
                    blocks: 0,
                    atime: SystemTime::now(),
                    mtime: SystemTime::now(),
                    ctime: SystemTime::now(),
                    kind: FileType::RegularFile,
                    perm: FILE_MODE,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 0,
                },
            })
        } else {
            Err(libc::ENOENT.into())
        }
    }

    async fn open(&self, _req: Request, inode: u64, flags: u32) -> Result<ReplyOpen> {
        if inode != PARENT_INODE && inode != FILE_INODE {
            return Err(libc::ENOENT.into());
        }

        Ok(ReplyOpen { fh: 0, flags })
    }

    async fn read(
        &self,
        _req: Request,
        inode: u64,
        _fh: u64,
        offset: u64,
        size: u32,
    ) -> Result<ReplyData> {
        if inode != FILE_INODE {
            return Err(libc::ENOENT.into());
        }

        if offset as usize >= CONTENT.len() {
            Ok(ReplyData {
                data: Box::new(b""),
            })
        } else {
            let mut data = &CONTENT.as_bytes()[offset as usize..];

            if data.len() > size as usize {
                data = &data[..size as usize];
            }

            Ok(ReplyData {
                data: Box::new(data),
            })
        }
    }

    async fn readdir(
        &self,
        _req: Request,
        inode: u64,
        _fh: u64,
        offset: i64,
    ) -> Result<ReplyDirectory> {
        if inode == FILE_INODE {
            return Err(libc::ENOTDIR.into());
        }

        if inode != PARENT_INODE {
            return Err(libc::ENOENT.into());
        }

        let entries = vec![
            DirectoryEntry {
                inode: PARENT_INODE,
                index: 1,
                kind: FileType::Directory,
                name: OsString::from("."),
            },
            DirectoryEntry {
                inode: PARENT_INODE,
                index: 2,
                kind: FileType::Directory,
                name: OsString::from(".."),
            },
            DirectoryEntry {
                inode: FILE_INODE,
                index: 3,
                kind: FileType::RegularFile,
                name: OsString::from(FILE_NAME),
            },
        ];

        Ok(ReplyDirectory {
            entries: Box::pin(stream::iter(entries.into_iter().skip(offset as usize))),
        })
    }

    async fn access(&self, _req: Request, inode: u64, _mask: u32) -> Result<()> {
        if inode != PARENT_INODE && inode != FILE_INODE {
            return Err(libc::ENOENT.into());
        }

        Ok(())
    }

    async fn readdirplus(
        &self,
        _req: Request,
        parent: u64,
        _fh: u64,
        offset: u64,
        _lock_owner: u64,
    ) -> Result<ReplyDirectoryPlus> {
        if parent == FILE_INODE {
            return Err(libc::ENOTDIR.into());
        }

        if parent != PARENT_INODE {
            return Err(libc::ENOENT.into());
        }

        let entries = vec![
            DirectoryEntryPlus {
                inode: PARENT_INODE,
                generation: 0,
                index: 1,
                kind: FileType::Directory,
                name: OsString::from("."),
                attr: FileAttr {
                    ino: PARENT_INODE,
                    generation: 0,
                    size: 0,
                    blocks: 0,
                    atime: SystemTime::now(),
                    mtime: SystemTime::now(),
                    ctime: SystemTime::now(),
                    kind: FileType::Directory,
                    perm: PARENT_MODE,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 0,
                },
                entry_ttl: TTL,
                attr_ttl: TTL,
            },
            DirectoryEntryPlus {
                inode: PARENT_INODE,
                generation: 0,
                index: 2,
                kind: FileType::Directory,
                name: OsString::from(".."),
                attr: FileAttr {
                    ino: PARENT_INODE,
                    generation: 0,
                    size: 0,
                    blocks: 0,
                    atime: SystemTime::now(),
                    mtime: SystemTime::now(),
                    ctime: SystemTime::now(),
                    kind: FileType::Directory,
                    perm: PARENT_MODE,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 0,
                },
                entry_ttl: TTL,
                attr_ttl: TTL,
            },
            DirectoryEntryPlus {
                inode: FILE_INODE,
                generation: 0,
                index: 3,
                kind: FileType::Directory,
                name: OsString::from(FILE_NAME),
                attr: FileAttr {
                    ino: FILE_INODE,
                    generation: 0,
                    size: CONTENT.len() as _,
                    blocks: 0,
                    atime: SystemTime::now(),
                    mtime: SystemTime::now(),
                    ctime: SystemTime::now(),
                    kind: FileType::RegularFile,
                    perm: FILE_MODE,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 0,
                },
                entry_ttl: TTL,
                attr_ttl: TTL,
            },
        ];

        Ok(ReplyDirectoryPlus {
            entries: Box::pin(stream::iter(entries.into_iter().skip(offset as usize))),
        })
    }

    async fn statsfs(&self, _req: Request, _inode: u64) -> Result<ReplyStatFs> {
        Ok(STATFS)
    }
}

#[async_std::main]
async fn main() {
    env_logger::init();
    let matches = App::new("mailfs")
        .arg(Arg::with_name("MOUNTPOINT")
              .index(1)
              .required(true))
        .get_matches();

    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };

    let mount_options = MountOptions::default().uid(uid).gid(gid).read_only(true);

    Session::new(mount_options)
        .mount_with_unprivileged(HelloWorld {}, matches.value_of("MOUNTPOINT").unwrap())
        .await
        .unwrap()
}
