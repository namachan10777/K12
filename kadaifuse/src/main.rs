use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::time::{Duration, SystemTime};

use futures_util::stream;

use async_trait::async_trait;
use fuse3::reply::{
    DirectoryEntry, DirectoryEntryPlus, ReplyAttr, ReplyData, ReplyDirectory, ReplyDirectoryPlus,
    ReplyEntry, ReplyOpen, ReplyStatFs,
};

use fuse3::{FileAttr, FileType, Filesystem, MountOptions, Request, Session};

use clap::{App, Arg};

const CONTENT: &str = "hello world\n";

const ROOT_INODE: u64 = 1;
const FILE_MODE: u16 = 0o444;
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

struct HelloWorld {
    ino_to_attr: HashMap<u64, FileAttr>,
    ino_to_text: HashMap<u64, String>,
    parent_to_entries: HashMap<u64, HashMap<String, u64>>,
}

struct AttrConfig {
    uid: u32,
    gid: u32,
    blksize: u32,
}

fn gen_dir_attr(cfg: &AttrConfig, ino: u64) -> FileAttr {
    FileAttr {
        ino,
        generation: 0,
        size: 0,
        blocks: 0,
        atime: SystemTime::now(),
        mtime: SystemTime::now(),
        ctime: SystemTime::now(),
        kind: FileType::Directory,
        perm: FILE_MODE,
        nlink: 0,
        uid: cfg.uid,
        gid: cfg.gid,
        rdev: 0,
        blksize: cfg.blksize,
    }
}

#[allow(dead_code)]
fn gen_mail_attr(cfg: &AttrConfig, ino: u64, size: u64) -> FileAttr {
    FileAttr {
        ino,
        generation: 0,
        size,
        blocks: ((size as i64 - 1) / cfg.blksize as i64) as u64,
        atime: SystemTime::now(),
        mtime: SystemTime::now(),
        ctime: SystemTime::now(),
        kind: FileType::RegularFile,
        perm: FILE_MODE,
        nlink: 0,
        uid: cfg.uid,
        gid: cfg.gid,
        rdev: 0,
        blksize: cfg.blksize,
    }
}

#[async_trait]
impl Filesystem for HelloWorld {
    async fn init(&self, _req: Request) -> fuse3::Result<()> {
        Ok(())
    }

    async fn destroy(&self, _req: Request) {}

    async fn lookup(&self, _req: Request, parent: u64, name: &OsStr) -> fuse3::Result<ReplyEntry> {
        self.parent_to_entries
            .get(&parent)
            .map(|entries| name.to_str().map(|fname| entries.get(fname)))
            .flatten()
            .flatten()
            .map(|ino| self.ino_to_attr.get(&ino))
            .flatten()
            .map(|attr| ReplyEntry {
                attr: *attr,
                ttl: TTL,
                generation: 0,
            })
            .ok_or_else(|| libc::ENOENT.into())
    }

    async fn getattr(
        &self,
        _req: Request,
        inode: u64,
        _fh: Option<u64>,
        _flags: u32,
    ) -> fuse3::Result<ReplyAttr> {
        self.ino_to_attr
            .get(&inode)
            .map(|attr| ReplyAttr {
                ttl: TTL,
                attr: *attr,
            })
            .ok_or_else(|| libc::ENOENT.into())
    }

    async fn open(&self, _req: Request, inode: u64, flags: u32) -> fuse3::Result<ReplyOpen> {
        if self.ino_to_attr.get(&inode).is_some() {
            Ok(ReplyOpen { fh: 0, flags })
        } else {
            Err(libc::ENOENT.into())
        }
    }

    async fn read(
        &self,
        _req: Request,
        inode: u64,
        _fh: u64,
        offset: u64,
        size: u32,
    ) -> fuse3::Result<ReplyData> {
        self.ino_to_text
            .get(&inode)
            .map(|text| {
                if offset as usize >= text.as_bytes().len() {
                    let empty = b"";
                    Box::new(&empty[..])
                } else {
                    let mut data = &CONTENT.as_bytes()[offset as usize..];

                    if data.len() > size as usize {
                        data = &data[..size as usize];
                    }
                    Box::new(data)
                }
            })
            .map(|data| ReplyData { data })
            .ok_or_else(|| libc::ENOENT.into())
    }

    async fn readdir(
        &self,
        _req: Request,
        inode: u64,
        _fh: u64,
        offset: i64,
    ) -> fuse3::Result<ReplyDirectory> {
        let mut basic = vec![
            DirectoryEntry {
                inode,
                index: 1,
                kind: FileType::Directory,
                name: OsString::from("."),
            },
            DirectoryEntry {
                inode: ROOT_INODE, // All directories can be a root or a mailbox.
                index: 2,
                kind: FileType::Directory,
                name: OsString::from(".."),
            },
        ];
        let entries = self.parent_to_entries.get(&inode).map(|entries| {
            entries
                .iter()
                .enumerate()
                .map(|(idx, (name, ino))| DirectoryEntry {
                    inode: *ino,
                    index: idx as u64 + 3,
                    kind: FileType::Directory,
                    name: OsString::from(name),
                })
                .collect::<Vec<DirectoryEntry>>()
        });
        if let Some(mut entries) = entries {
            basic.append(&mut entries);
            Ok(ReplyDirectory {
                entries: Box::pin(stream::iter(entries.into_iter().skip(offset as usize))),
            })
        } else {
            Err(libc::ENOENT.into())
        }
    }

    async fn access(&self, _req: Request, inode: u64, _mask: u32) -> fuse3::Result<()> {
        if self.ino_to_attr.get(&inode).is_some() {
            Ok(())
        } else {
            Err(libc::ENOENT.into())
        }
    }

    // TODO code duplicated with readdir
    async fn readdirplus(
        &self,
        _req: Request,
        inode: u64,
        _fh: u64,
        offset: u64,
        _lock_owner: u64,
    ) -> fuse3::Result<ReplyDirectoryPlus> {
        let mut basic = vec![
            DirectoryEntryPlus {
                inode,
                index: 1,
                kind: FileType::Directory,
                name: OsString::from("."),
                generation: 0,
                attr: *self.ino_to_attr.get(&inode).unwrap(),
                attr_ttl: TTL,
                entry_ttl: TTL,
            },
            DirectoryEntryPlus {
                inode: ROOT_INODE, // All directories can be a root or a mailbox.
                index: 2,
                attr: *self.ino_to_attr.get(&ROOT_INODE).unwrap(),
                attr_ttl: TTL,
                entry_ttl: TTL,
                generation: 0,
                kind: FileType::Directory,
                name: OsString::from(".."),
            },
        ];
        let entries = self.parent_to_entries.get(&inode).map(|entries| {
            entries
                .iter()
                .enumerate()
                .map(|(idx, (name, ino))| DirectoryEntryPlus {
                    inode: *ino,
                    index: idx as u64 + 3,
                    kind: FileType::Directory,
                    name: OsString::from(name),
                    generation: 0,
                    attr: *self.ino_to_attr.get(ino).unwrap(),
                    attr_ttl: TTL,
                    entry_ttl: TTL,
                })
                .collect::<Vec<DirectoryEntryPlus>>()
        });
        if let Some(mut entries) = entries {
            basic.append(&mut entries);
            Ok(ReplyDirectoryPlus {
                entries: Box::pin(stream::iter(entries.into_iter().skip(offset as usize))),
            })
        } else {
            Err(libc::ENOENT.into())
        }
    }

    async fn statsfs(&self, _req: Request, _inode: u64) -> fuse3::Result<ReplyStatFs> {
        Ok(STATFS)
    }
}

#[async_std::main]
async fn main() {
    env_logger::init();
    let matches = App::new("mailfs")
        .arg(Arg::with_name("MOUNTPOINT").index(1).required(true))
        .arg(
            Arg::with_name("DOMAIN")
                .required(true)
                .short("d")
                .long("domain"),
        )
        .arg(
            Arg::with_name("USERNAME")
                .required(true)
                .short("n")
                .long("username"),
        )
        .arg(
            Arg::with_name("PASSWORD")
                .required(true)
                .short("p")
                .long("password"),
        )
        .get_matches();

    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };

    let mount_options = MountOptions::default().uid(uid).gid(gid).read_only(true);

    let mut ino_to_attr = HashMap::new();
    let cfg = AttrConfig {
        blksize: 4096,
        uid: 501, // FIXME
        gid: 20,  // FIXME
    };
    ino_to_attr.insert(ROOT_INODE, gen_dir_attr(&cfg, ROOT_INODE));
    let ino_to_text = HashMap::new();
    let mut parent_to_entries = HashMap::new();
    parent_to_entries.insert(ROOT_INODE, HashMap::new());
    let fs = HelloWorld {
        ino_to_text,
        ino_to_attr,
        parent_to_entries,
    };

    Session::new(mount_options)
        .mount_with_unprivileged(fs, matches.value_of("MOUNTPOINT").unwrap())
        .await
        .unwrap()
}
