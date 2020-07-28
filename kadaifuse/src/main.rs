use fuse::{
    Filesystem,
    ReplyEntry,
    ReplyAttr,
    ReplyData,
    ReplyDirectory,
    ReplyOpen,
    Request,
    FileAttr,
};

use std::collections::HashMap;
use std::ffi::OsStr;
use time::Timespec;
use log::info;
const ENOENT: i32 = 44;

struct MailFs {
    file_handle_count: u64,
    dir_handle_count: u64,
    file_handle_map: HashMap<u64, u64>,
    dir_handle_map: HashMap<u64, u64>,

    box_to_ino: HashMap<String, (u64, FileAttr)>,
    boxes: HashMap<u64, HashMap<String, u64>>,

    files: HashMap<u64, (FileAttr, String)>,
    root_ino: u64,
}

const DEFAULT_TS : Timespec = Timespec {
    sec: 1,
    nsec: 0,
};

impl Filesystem for MailFs {
    fn open(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        info!("open {}", ino);
        if self.files.get(&ino).is_some() {
            self.file_handle_count += 1;
            self.file_handle_map.insert(self.file_handle_count, ino);
            reply.opened(self.file_handle_count, flags);
        }
        else {
            reply.error(ENOENT)
        }
    }

    fn opendir(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        info!("opendir {}", ino);
        if self.files.get(&ino).is_some() {
            self.dir_handle_count += 1;
            self.dir_handle_map.insert(self.dir_handle_count, ino);
            reply.opened(self.dir_handle_count, flags);
        }
        else {
            reply.error(ENOENT)
        }
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        info!("lookup {:?}", name);
        if parent == self.root_ino {
            if let Some((_, attr)) = self.box_to_ino.get(name.to_str().unwrap()) {
                reply.entry(&DEFAULT_TS, attr, 0);
            }
            else {
                reply.error(ENOENT)
            }
        }
        else if let Some(mailbox) = self.boxes.get(&parent){
            if let Some(ino) = mailbox.get(name.to_str().unwrap()) {
                if let Some((attr, _)) = self.files.get(ino) {
                    reply.entry(&DEFAULT_TS, attr, 0);
                }
            }
        }
        else {
            reply.error(ENOENT)
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        if let Some((attr, _)) = self.files.get(&ino) {
            info!("getattr {:?}", attr);
            reply.attr(&DEFAULT_TS, attr);
        }
        else {
            reply.error(ENOENT)
        }
    }

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, size: u32, reply: ReplyData) {
        info!("read {}", ino);
        if let Some((_, data)) = self.files.get(&ino) {
            if (data.len() as i64) < (size as i64 - offset) {
                reply.data(&data.as_bytes()[(offset as usize)..])
            }
            else {
                reply.data(&data.as_bytes()[(offset as usize)..(offset as usize)+(size as usize)])
            }
        }
        else {
            reply.error(ENOENT)
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        reply.add(1, 1, fuse::FileType::Directory, ".");
        reply.add(1, 2, fuse::FileType::Directory, "..");
        if ino == self.root_ino {
            for (i, (box_name, (box_ino, _))) in self.box_to_ino.iter().enumerate().skip(offset as usize) {
                reply.add(*box_ino, (i as i64) + 3, fuse::FileType::Directory, box_name);
            }
            reply.ok()
        }
        else {
            if let Some(dir) = self.boxes.get(&ino) {
                for (box_name, box_ino) in dir {
                    reply.add(*box_ino, 0, fuse::FileType::RegularFile, box_name);
                }
                reply.ok()
            }
            else {
                reply.error(ENOENT);
            }
        }
    }
}

fn main() {
    let root_attr = fuse::FileAttr {
        ino: 1,
        size: 0,
        blocks: 0,
        atime: Timespec::new(0, 0),
        mtime: Timespec::new(0, 0),
        ctime: Timespec::new(0, 0),
        crtime: Timespec::new(0, 0),
        kind: fuse::FileType::Directory,
        perm: 0o755,
        nlink: 2,
        uid: 501,
        gid: 20,
        rdev: 0,
        flags: 0,
    };
    env_logger::init();
    let matches = clap::App::new("mailfs")
        .arg(clap::Arg::with_name("MOUNTPOINT").required(true).index(1))
        .get_matches();
    let options = ["-o", "ro", "-o", "fsname=mailfs"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    let mut files = HashMap::new();
    files.insert(1, (root_attr, String::new()));
    let boxes = HashMap::new();
    let box_to_ino = HashMap::new();
    let fs = MailFs {
        file_handle_count: 0,
        file_handle_map: HashMap::new(),
        dir_handle_count: 0,
        dir_handle_map: HashMap::new(),
        box_to_ino,
        boxes,
        files,
        root_ino: 1,
    };
    fuse::mount(fs, &matches.value_of("MOUNTPOINT").unwrap(), &options).unwrap();
}
