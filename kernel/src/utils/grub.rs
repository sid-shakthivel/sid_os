/*
    Grub 2 (GNU bootloader) is a bootloader which uses a header file to configure options
    Grub loads a number of modules(user programs) into certain memory locations which need to be mapped into user pages
    Grub emulates VGA card
    BGA (Bochs Graphic Updator) is accessible via 2 ports (index, data) in which it's possible to enable/disable VBE extentions
    Includes changing screen resolution, dit depth | Latest version is 0xB0C5
*/

use core::panic;

use crate::{
    multitask::{self, PROCESS_MANAGER},
    print_serial,
};

use super::{
    multiboot2_test::MultibootBootInfo,
    ports::{inpw, outpw},
};

const VBE_DISPI_IOPORT_INDEX: u16 = 0x01CE;
const VBE_DISPI_IOPORT_DATA: u16 = 0x01CF;
const VBE_DISPI_INDEX_ID: u16 = 0;
const VBE_DISPI_INDEX_XRES: u16 = 1;
const VBE_DISPI_INDEX_YRES: u16 = 2;
const VBE_DISPI_INDEX_BPP: u16 = 3;
const VBE_DISPI_INDEX_ENABLE: u16 = 4;
const VBE_DISPI_INDEX_BANK: u16 = 5;
const VBE_DISPI_INDEX_VIRT_WIDTH: u16 = 6;
const VBE_DISPI_INDEX_VIRT_HEIGHT: u16 = 7;
const VBE_DISPI_INDEX_X_OFFSET: u16 = 8;
const VBE_DISPI_INDEX_Y_OFFSET: u16 = 9;
const VBE_DISPI_LFB_ENABLED: u16 = 0x40;

pub fn initalise_userland(multiboot_info: &MultibootBootInfo) {
    let mut count: u32 = 0;

    // for tag in multiboot_info.tags() {
    //     count += 1;
    //     print_serial!("{}\n", tag.typ);
    // }

    print_serial!("nmber of totoal tags = {}\n", count);

    for tag in multiboot_info.get_module_tags() {
        // All modules are programs (so far)

        let module_addr = tag.mod_start as usize;
        let module_len = (tag.mod_end - tag.mod_start) as usize;

        print_serial!(
            "This is the module start address 0x{:x} and this is the size 0x{:x}\n",
            module_addr,
            module_len
        );

        PROCESS_MANAGER.lock().add_process(
            multitask::ProcessPriority::High,
            0,
            (module_addr, module_len),
        );
        PROCESS_MANAGER.free();
    }
}

pub fn get_end_of_memory(multiboot_info: &MultibootBootInfo) -> usize {
    let mut end_memory: usize = 0;
    let mmap_tag = multiboot_info.get_memory_map_tag().expect("Expected mmap");
    for tag in mmap_tag.get_available_mmap_entries() {
        print_serial!(
            "Memory Map Entry:\n Start Address: 0x{:x}\n",
            tag.start_address()
        );
        print_serial!("End Address: 0x{:x}\n", tag.end_address());
        end_memory = tag.end_address()
    }
    end_memory
}

pub fn bga_set_video_mode() {
    if !is_bga_available() {
        panic!("BGA is not available");
    }
    write_bga_register(VBE_DISPI_INDEX_ENABLE, 0x00); // To modify contents of other registers, VBE extensions must be disabled
    write_bga_register(VBE_DISPI_INDEX_XRES, 1024);
    write_bga_register(VBE_DISPI_INDEX_YRES, 768);
    write_bga_register(VBE_DISPI_INDEX_BPP, 0x20);
    write_bga_register(VBE_DISPI_INDEX_ENABLE, 0x01);
    write_bga_register(VBE_DISPI_INDEX_BANK, VBE_DISPI_LFB_ENABLED | 0x1); // Linear frame buffer
}

fn write_bga_register(index: u16, value: u16) {
    outpw(VBE_DISPI_IOPORT_INDEX, index);
    outpw(VBE_DISPI_IOPORT_DATA, value);
}

fn read_bga_register(index: u16) -> u16 {
    outpw(VBE_DISPI_IOPORT_INDEX, index);
    return inpw(VBE_DISPI_IOPORT_DATA);
}

fn is_bga_available() -> bool {
    return read_bga_register(VBE_DISPI_INDEX_ID) == 0xB0C5;
}
