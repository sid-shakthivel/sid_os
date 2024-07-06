use core::intrinsics::size_of;

use super::fat::{convert_sector_to_bytes, get_next_cluster, FileEntry, LongFileEntry};
use crate::print_serial;
use crate::utils::{self, string};
use crate::{ds::tree::TreeNode, utils::spinlock::Lock};

#[derive(Copy, PartialEq, Clone, Debug)]
pub enum FileType {
    File,
    Directory,
    Syslink,
}

#[derive(Debug, Clone, Copy)]
struct File {
    name: &'static str,
    size: usize,
    flags: usize,
    id: usize,
    offset: usize,
    current_cluster: usize,
    f_type: FileType,
}

impl File {
    pub const fn new(
        name: &'static str,
        size: usize,
        f_type: FileType,
        current_cluster: usize,
    ) -> File {
        File {
            name,
            size,
            flags: 0,
            id: 0,
            offset: 0,
            current_cluster,
            f_type,
        }
    }
}

struct Vfs {
    fat_addr: usize,
    rd_addr: usize,
    ds_addr: usize,
    root: TreeNode<File>,
}

impl Vfs {
    pub const fn new() -> Vfs {
        Vfs {
            fat_addr: 0,
            rd_addr: 0,
            ds_addr: 0,
            root: TreeNode::new_const(),
        }
    }

    fn init(&mut self, values: (usize, usize, usize)) {
        self.fat_addr = values.0;
        self.rd_addr = values.1;
        self.ds_addr = values.2;

        self.root = TreeNode::new(File::new("root", 0, FileType::Directory, 0));

        self.build_vfs();
    }

    fn build_vfs(&mut self) {
        // Iterate through root directory to build initial tree
        let mut addr = self.rd_addr as *const u8;

        unsafe {
            while (*addr) != 0 {
                let file = &*(self.rd_addr as *const FileEntry);
                let filename_start = string::convert_utf8_to_trimmed_string(&file.filename);
                addr = addr.add(size_of::<FileEntry>());
            }
        }
    }

    fn iterate_dir(&mut self, mut addr: *const u8) {
        unsafe {
            while (*addr) != 0 {
                let file = &*(self.rd_addr as *const FileEntry);

                let file_type = match file.attributes {
                    0x10 => FileType::Directory,
                    0x20 => FileType::File,
                    _ => panic!("Error: Unknown file type encountered"),
                };

                let file = File::new(
                    string::convert_utf8_to_trimmed_string(&file.filename),
                    file.size as usize,
                    file_type,
                    file.cluster_low as usize,
                );

                addr = addr.add(size_of::<FileEntry>());
            }
        }
    }

    pub fn find(&mut self, filename: &str) -> Option<File> {
        fn print_node<T: core::fmt::Debug>(node: &TreeNode<T>) {
            print_serial!("{:?}", node.payload);
        }

        self.root.traverse(print_node);
        None
    }
}

pub static VFS: Lock<Vfs> = Lock::new(Vfs::new());
