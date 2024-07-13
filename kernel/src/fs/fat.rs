use core::ffi::CStr;
use core::{fmt::Error, mem::size_of, ptr};

use crate::fs::fat;
use crate::utils::string;
use crate::{memory::allocator::kmalloc, print_serial};

const BYTES_PER_SECTOR: usize = 512;
pub const BYTES_PER_CLUSTER: usize = 2048;
const BYTES_PER_FAT: usize = 10240;

// Boot record occupies one sector and is at the start
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct BiosParameterBlock {
    jmp: [u8; 3],
    oem: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sector_count: u16,
    table_count: u8,
    root_entry_count: u16,
    sector_count_16: u16,
    media_type: u8,
    table_size_16: u16,     // Number of sectors per FAT
    sectors_per_track: u16, // Number of sectors per track
    head_count: u16,
    hidden_sector_count: u32,
    large_sector_count: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct ExtendedBootRecord {
    drive_number: u8,
    nt_flags: u8,
    signature: u8,
    serial: u32,
    volume_label: [u8; 11],
    system_ud_string: u64,
    bootcode: [u8; 448],
    bootable_partition_signature: u16,
}

// Stores information on where a file's data/folder are stored on disk along with name, size, creation
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct FileEntry {
    pub filename: [u8; 8],
    pub ext: [u8; 3],
    pub attributes: u8, // Could be LFN, Directory, Archive
    unused: [u8; 8],    // Reserved for windows NT
    cluster_high: u16,  // Always 0
    time: u16,
    date: u16,
    pub cluster_low: u16,
    pub size: u32, // (In bytes)
}

// These always have a regular entry as well, and these are placed before the standard entry and hold extra data
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct LongFileEntry {
    order: u8,             // Since there could be number of LFE's, order is important
    name_start: [u16; 5],  // First 5 characters
    attribute: u8,         // 0x0F
    long_entry_type: u8,   // 0x00
    checksum: u8,          // Checksum generated fo the short file name
    name_middle: [u16; 6], // Next 6 characters
    zero: u16,             // Always 0
    name_end: [u16; 2],    // Final 2 characters
}

impl FileEntry {
    pub fn new() -> FileEntry {
        FileEntry {
            filename: [0; 8],
            ext: [0; 3],
            attributes: 0,
            unused: [0; 8],
            cluster_high: 0,
            time: 0,
            date: 0,
            cluster_low: 0,
            size: 0,
        }
    }
}

impl BiosParameterBlock {
    pub fn verify(&self) {
        assert!(
            self.jmp[0] == 0xEB && self.jmp[1] == 0x3C && self.jmp[2] == 0x90,
            "Error: Invalid JMP sequence"
        );

        assert!(
            self.bytes_per_sector == BYTES_PER_SECTOR as u16,
            "Error: Bytes per sector is not 512"
        );

        assert!(
            self.sectors_per_cluster == 4,
            "Error: Sectors per cluster is not 4"
        );

        assert!(self.table_count == 2, "Error: Table count is not 2");

        assert!(self.table_size_16 == 20, "Error: Table size is not 20");

        // field is set if there are more than 65535 sectors in the volume this may cause problems *CHECK*
        assert!(
            self.large_sector_count == 0,
            "Error: Invalid Large Sector Count"
        );
    }
}

impl ExtendedBootRecord {
    pub fn verify(&self) {
        assert!(
            self.signature == 0x29 || self.signature == 0x28,
            "Error: Invalid Signature"
        );

        assert!(
            self.bootable_partition_signature == 0xAA55,
            "Error: Invalid Bootable Signature"
        );
    }
}

pub fn convert_sector_to_bytes(sector: usize) -> usize {
    return sector * BYTES_PER_SECTOR;
}

pub fn get_next_cluster(fat_addr: usize, active_cluster: usize) -> Option<(usize)> {
    let fat_offset = active_cluster * 2;

    let next_cluster = read_fat(fat_addr, fat_offset) as usize;

    match next_cluster {
        0x00 => panic!("Error: Empty cluster?"),
        0xFFF7 => panic!("Error: Bad cluster when reading cluster!"), // Indicates bad cluster
        0xFFF8..=0xFFFF => None, // Indicates the whole file has been read
        _ => Some(next_cluster), // Gives next cluster number
    }
}

pub fn write_fat(fat_addr: usize, cluster: usize, next_cluster: usize) {
    let fat_offset = cluster * 2;
    let fat = unsafe { &mut *(fat_addr as *mut [u8; 512]) };
    fat[fat_offset] = (next_cluster & 0x00FF) as u8;
    fat[fat_offset + 1] = ((next_cluster & 0xFF00) >> 8) as u8;
}

pub fn find_free_cluster(fat_addr: usize) -> Option<usize> {
    let fat = unsafe { &mut *(fat_addr as *mut [u8; 512]) };
    for i in 0..(BYTES_PER_FAT / 2) {
        if ((fat[i + 1] as u16) << 8 | (fat[i] as u16)) == 0 {
            fat[i] = 0xFF;
            fat[i + 1] = 0xFF;
            return Some(i as usize);
        }
    }
    None
}

fn read_fat(fat_addr: usize, fat_offset: usize) -> u16 {
    let sector_num = fat_offset / 512;
    let byte_offset = fat_offset % 512;

    let fat = unsafe { &*((fat_addr + convert_sector_to_bytes(sector_num)) as *const [u8; 512]) };
    return ((fat[byte_offset + 1] as u16) << 8) | (fat[byte_offset] as u16);
}

pub fn get_sector_from_cluster(sector_addr: usize, cluster_num: usize) -> *mut u8 {
    (((cluster_num - 2) * 4) + sector_addr) as *mut u8
}

pub fn init(start_addr: usize) -> (usize, usize, usize) {
    let bpb = unsafe { &*(start_addr as *const BiosParameterBlock) };

    print_serial!("{:?}\n", bpb);

    let ebr = unsafe {
        &*((start_addr as *mut u8).offset(size_of::<BiosParameterBlock>() as isize)
            as *const ExtendedBootRecord)
    };

    bpb.verify();
    ebr.verify();

    let fat_addr = start_addr + convert_sector_to_bytes(bpb.reserved_sector_count as usize);

    let rd_sector_num: usize = (bpb.reserved_sector_count as usize)
        + ((bpb.table_count as usize) * bpb.table_size_16 as usize);

    let rd_addr: usize = start_addr + convert_sector_to_bytes(rd_sector_num);

    let rd_size: usize = ((((bpb.root_entry_count) * 32) + (bpb.bytes_per_sector - 1))
        / bpb.bytes_per_sector) as usize;

    let ds_addr: usize = convert_sector_to_bytes(rd_size) + rd_addr;

    return (fat_addr, rd_addr, ds_addr);
}
