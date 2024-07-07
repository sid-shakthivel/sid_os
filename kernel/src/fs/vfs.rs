use core::intrinsics::size_of;

use super::fat::{
    convert_sector_to_bytes, get_next_cluster, get_sector_from_cluster, FileEntry, LongFileEntry,
};
use crate::fs::fat::BYTES_PER_CLUSTER;
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
    }

    fn iterate_dir(&mut self, mut addr: *const u8, current_node: &mut TreeNode<File>) {
        const ENTRIES_PER_CLUSTER: usize = BYTES_PER_CLUSTER / size_of::<FileEntry>();

        for i in 0..ENTRIES_PER_CLUSTER {
            match unsafe { *addr } {
                0 => break,
                0xE5 => continue,
                _ => {}
            }

            let file = unsafe { &*(self.rd_addr as *const FileEntry) };

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

            match file_type {
                FileType::Directory => {
                    current_node.add_child(TreeNode::new(file));

                    self.iterate_dir(
                        get_sector_from_cluster(file.current_cluster, file.current_cluster)
                            as *const u8,
                        current_node.children.get_last().unwrap(),
                    );
                }
                FileType::File => {
                    current_node.add_child(TreeNode::new(file));
                }
                _ => {}
            }

            addr = unsafe { addr.add(size_of::<FileEntry>()) };
        }

        // Note that the cluster number inside the file is not updated
        let file = unsafe { &mut *current_node.payload };
        if let Some(addr) = get_next_cluster(self.fat_addr, file.current_cluster) {
            self.iterate_dir(addr as *const u8, current_node);
        }
    }

    fn read_file(&self, file: &File, buffer: *mut u8) {
        if file.f_type == FileType::Directory {
            return;
        }

        let mut current_cluster = Some(file.current_cluster);

        let mut size_left = file.size;

        while let Some(cluster) = current_cluster {
            let cluster_addr = get_sector_from_cluster(self.fat_addr, cluster);
            let bytes_to_copy = size_left.max(BYTES_PER_CLUSTER);
            unsafe {
                core::ptr::copy(cluster_addr as *mut u8, buffer, bytes_to_copy);
            }

            size_left -= bytes_to_copy;
            current_cluster = get_next_cluster(self.fat_addr, cluster);
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
