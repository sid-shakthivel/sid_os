- Add support for grub modules
- Parse the multiboot properly
- Bochs breakpoint is xchg bx, bx
- https://wiki.osdev.org/Exceptions
- https://wiki.osdev.org/Programmable_Interval_Timer
- Must consider way to handle handle cr3 register (deep clone to start with)
- Replace the free stack within the pfa
- When popping should really kfree the address

http://www.jamesmolloy.co.uk/tutorial_html/9.-Multitasking.html
http://www.brokenthorn.com/Resources/OSDev25.html
https://is.muni.cz/el/fi/jaro2018/PB173/um/05_writing_a_very_small_os_kernel/osdev.pdf

/// An enum of possible reported region types.
/// Inside the Multiboot2 spec this is kind of hidden
/// inside the implementation of `struct multiboot_mmap_entry`.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u32)]
pub enum MemoryAreaType {
    /// Available memory free to be used by the OS.
    Available = 1,

    /// A reserved area that must not be used.
    Reserved = 2,

    /// Usable memory holding ACPI information.
    AcpiAvailable = 3,

    /// Reserved memory which needs to be preserved on hibernation.
    /// Also called NVS in spec, which stands for "Non-Volatile Sleep/Storage",
    /// which is part of ACPI specification.
    ReservedHibernate = 4,

    /// Memory which is occupied by defective RAM modules.
    Defective = 5,
}