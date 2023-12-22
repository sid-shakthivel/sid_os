use lazy_static::lazy_static;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::tss::TaskStateSegment;

pub static mut TSS: TaskStateSegment = TaskStateSegment::new();

use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        unsafe {
            let mut gdt = GlobalDescriptorTable::new();
            let kernel_code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
            let kernel_data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
            let _user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
            let _user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
            let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
            (
                gdt,
                Selectors {
                    kernel_code_selector,
                    kernel_data_selector,
                    tss_selector,
                },
            )
        }
    };
}

struct Selectors {
    kernel_code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
    kernel_data_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS, DS, ES, FS, GS, SS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        // Reload all segment registers
        CS::set_reg(GDT.1.kernel_code_selector);
        SS::set_reg(GDT.1.kernel_data_selector);
        DS::set_reg(GDT.1.kernel_data_selector);
        ES::set_reg(GDT.1.kernel_data_selector);
        FS::set_reg(GDT.1.kernel_data_selector);
        GS::set_reg(GDT.1.kernel_data_selector);
        load_tss(GDT.1.tss_selector);
    }
}
