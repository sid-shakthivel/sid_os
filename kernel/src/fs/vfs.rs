use core::intrinsics::size_of;

use crate::fs::fat::{self, BYTES_PER_CLUSTER};
use crate::utils::{self, string};
use crate::{ds::tree::TreeNode, utils::spinlock::Lock};
use crate::{either, print_serial};

#[derive(Copy, PartialEq, Clone, Debug)]
pub enum FileType {
    File,
    Directory,
    Syslink,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct File {
    name: &'static str,
    pub size: usize,
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

pub struct Vfs {
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

    pub fn init(&mut self, values: (usize, usize, usize)) {
        self.fat_addr = values.0;
        self.rd_addr = values.1;
        self.ds_addr = values.2;

        self.root = TreeNode::new(File::new("root", 0, FileType::Directory, 0));

        let addr = self.rd_addr as *const u8;
        let current_node = &mut self.root.clone();
        self.build_vfs(addr, current_node);
        self.root = current_node.clone();
    }

    fn build_vfs(&mut self, mut addr: *const u8, current_node: &mut TreeNode<File>) {
        const ENTRIES_PER_CLUSTER: usize = BYTES_PER_CLUSTER / size_of::<fat::FileEntry>();

        for i in 0..ENTRIES_PER_CLUSTER {
            match unsafe { *addr } {
                0 => return,
                0xE5 => {
                    addr = unsafe { addr.add(size_of::<fat::FileEntry>()) };
                    continue;
                }
                _ => {}
            }

            let file_entry = unsafe { &*(addr as *const fat::FileEntry) };

            let file_type = match file_entry.attributes {
                0x10 => FileType::Directory,
                0x20 => FileType::File,
                _ => panic!("Error: Unknown file type encountered"),
            };

            let filename = string::convert_utf8_to_trimmed_string(&file_entry.filename);
            let ext = string::convert_utf8_to_trimmed_string(&file_entry.ext);

            if filename.starts_with(".") {
                addr = unsafe { addr.add(size_of::<fat::FileEntry>()) };
                continue;
            }

            let mut file = File::new(
                filename,
                file_entry.size as usize,
                file_type,
                file_entry.cluster_low as usize,
            );

            match file_type {
                FileType::Directory => {
                    current_node.add_child(TreeNode::new(file));

                    self.build_vfs(
                        fat::get_sector_from_cluster(self.ds_addr, file.current_cluster),
                        current_node.children.get_last().unwrap(),
                    );
                }
                FileType::File => {
                    unsafe {
                        for byte in FILE_NAME_BUFFER.iter_mut() {
                            *byte = 0;
                        }
                    }

                    let combined_str = unsafe {
                        match string::concatenate_filename_ext(filename, ext, &mut FILE_NAME_BUFFER)
                        {
                            Ok(result) => result,
                            Err(error) => {
                                panic!("Error: {}", error);
                            }
                        }
                    };

                    file.name = combined_str;

                    current_node.add_child(TreeNode::new(file));
                }
                _ => {}
            }

            addr = unsafe { addr.add(size_of::<fat::FileEntry>()) };
        }

        // Note that the cluster number inside the file is not updated
        let file = unsafe { &mut *current_node.payload };
        if let Some(addr) = fat::get_next_cluster(self.fat_addr, file.current_cluster) {
            self.build_vfs(addr as *const u8, current_node);
        }
    }

    fn delete_file_from_disk(&mut self, file: &File, parent: &File) {
        let mut addr = fat::get_sector_from_cluster(self.fat_addr, parent.current_cluster);

        if parent.name == "root" {
            addr = self.ds_addr as *mut u8;
        }

        for i in 0..(BYTES_PER_CLUSTER / size_of::<fat::FileEntry>()) {
            let file_entry = unsafe { &*(addr as *const fat::FileEntry) };

            if file_entry.cluster_low == file.current_cluster as u16 {
                unsafe {
                    core::ptr::write(addr as *mut u8, 0xE5);
                }
            }

            addr = unsafe { addr.add(size_of::<fat::FileEntry>()) };
        }

        // Rebuild filesystem

        let addr = self.rd_addr as *const u8;
        let current_node = &mut self.root.clone();
        self.build_vfs(addr, current_node);
        self.root = current_node.clone();
    }

    fn write_file_to_disk(&mut self, file: &File, parent: &File) {
        let mut addr = fat::get_sector_from_cluster(self.fat_addr, parent.current_cluster);

        if parent.name == "root" {
            addr = self.ds_addr as *mut u8;
        }

        for i in 0..(BYTES_PER_CLUSTER / size_of::<fat::FileEntry>()) {
            unsafe {
                if (*addr) == 0 {
                    // Write new file entry at this address
                    let new_file_entry = fat::FileEntry::new();
                    core::ptr::write(addr as *mut fat::FileEntry, new_file_entry);
                }
            }

            addr = unsafe { addr.add(size_of::<fat::FileEntry>()) };
        }

        let addr = self.rd_addr as *const u8;
        let current_node = &mut self.root.clone();
        self.build_vfs(addr, current_node);
        self.root = current_node.clone();
    }

    pub fn write_file(&self, file: &mut File, mut buffer: *mut u8, length: usize) {
        if file.f_type == FileType::Directory {
            return;
        }

        let mut size_left = length;
        let mut current_cluster = Some(file.current_cluster);
        let mut previous_cluster = file.current_cluster;

        // TODO: Need to update the file_entry thing (could be a property of the file)
        file.size = either!(file.size > length => file.size; length);

        while size_left > 0 {
            let bytes_to_copy = size_left.min(BYTES_PER_CLUSTER);

            if let Some(cluster) = current_cluster {
                let cluster_addr = fat::get_sector_from_cluster(self.fat_addr, cluster);
                unsafe {
                    core::ptr::copy(cluster_addr, buffer, bytes_to_copy);
                    buffer = buffer.add(bytes_to_copy);
                }
                previous_cluster = cluster;
                current_cluster = fat::get_next_cluster(self.fat_addr, cluster);
            } else {
                // Search FAT for unallocated cluster
                if let Some(next_cluster) = fat::find_free_cluster(self.fat_addr) {
                    // Write for previous entry to point to new cluster
                    fat::write_fat(self.fat_addr, previous_cluster, next_cluster);

                    let cluster_addr = fat::get_sector_from_cluster(self.fat_addr, next_cluster);
                    unsafe {
                        core::ptr::copy(cluster_addr, buffer, bytes_to_copy);
                        buffer = buffer.add(bytes_to_copy);
                    }

                    current_cluster = Some(next_cluster);
                }
            }
            size_left -= BYTES_PER_CLUSTER;
        }
    }

    pub fn read_file(&self, file: &File, buffer: *mut u8, length: usize) {
        if file.f_type == FileType::Directory {
            return;
        }

        let mut current_cluster = Some(file.current_cluster);

        let mut size_left = length;

        while let Some(cluster) = current_cluster {
            let cluster_addr = fat::get_sector_from_cluster(self.fat_addr, cluster);
            let bytes_to_copy = size_left.max(BYTES_PER_CLUSTER);
            unsafe {
                core::ptr::copy(cluster_addr, buffer, bytes_to_copy);
            }

            size_left -= bytes_to_copy;
            current_cluster = fat::get_next_cluster(self.fat_addr, cluster);
        }
    }

    pub fn open(&self, filepath: &str) -> File {
        let is_absolute = filepath.starts_with("/");

        assert!(is_absolute, "Error: Filename must be absolute");

        let cleaned_filepath: &str = &filepath[1..filepath.len()];
        let mut filepath_components = cleaned_filepath.split("/");

        let mut current_node = self.root.clone();
        for component in filepath_components {
            if let Some(node) = self.find(component, &current_node) {
                current_node = node;
            } else {
                panic!("Error: File not found");
            }
        }

        let file = unsafe { &*current_node.payload };
        return file.clone();
    }

    fn find(&self, filename: &str, current_node: &TreeNode<File>) -> Option<TreeNode<File>> {
        for child in current_node.children.iter() {
            let file = unsafe { &*child.payload };
            if file.name == filename {
                return Some(child.clone());
            }
        }

        None
    }

    pub fn print(&self) {
        fn print_node(node: &TreeNode<File>) {
            let file = unsafe { &*node.payload };
            print_serial!("{:?}\n", file);
        }

        self.root.traverse(&print_node);
    }
}

pub static VFS: Lock<Vfs> = Lock::new(Vfs::new());
static mut FILE_NAME_BUFFER: [u8; 32] = [0u8; 32];
