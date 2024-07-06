Remember:
- Bochs breakpoint is xchg bx, bx
- info tab specifies v_addr then p_addr
- To print a value in a packed struct do the following:
```
    let ptr = core::ptr::addr_of!(TSS.privilege_stack_table[0]);
    let val = unsafe { ptr.read_unaligned() };
```

Now: 
- FAT filesystem driver
- Figure out way to concatenate strings
- Syscalls with file system
- IPC with message queues
- Wrapping sub zero (for window manager)
- Events queue for eventual usermode stuff
- Add back nicer window design

Where does font= come from? not in makefile

Format of TGA files:
- Do not use RLE compression
- Top left

int send_message(int cpid, int pid, char *ptr)
{
    int64_t result;
    asm volatile("mov %1, %%ebx \n\t\
        mov %2, %%ecx \n\t\
        mov %3, %%edx \n\t\
        mov $20, %%eax \n\t\
        int $0x80 \n\t\
        "
                 : "=r"(result)
                 : "r"(cpid), "r"(pid), "m"(ptr));
    return (int)result;
}

Refactoring:
- tga file stuff

New:
- Sleep syscall thing
- Switch

Later:
- Add more comments everywhere

Useful articles:
https://fejlesztek.hu/create-a-fat-file-system-image-on-linux/
https://github.com/thethumbler/Aquila?tab=readme-ov-file
https://archive.is/KmqPR
http://www.brokenthorn.com/Resources/OSDev25.html
https://is.muni.cz/el/fi/jaro2018/PB173/um/05_writing_a_very_small_os_kernel/osdev.pdf
http://dmitrysoshnikov.com/compilers/writing-a-memory-allocator/
https://wiki.osdev.org/Brendan%27s_Multi-tasking_Tutorial
https://jmarlin.github.io/wsbe/

export CC=x86_64-elf-gcc CFLAGS='--target=x86_64-pc-none-elf -march=x86_64 -DSYSCALL_NO_TLS' LDFLAGS='-fuse-ld=lld' 
export CXX=x86_64-elf-g++
export AR=x86_64-elf-ar
export RANLIB=x86_64-elf-ranlib
export LD=x86_64-elf-ld

ln -s /usr/local/bin/x86_64-elf-ar x86_64-sidos-ar       
ln -s /usr/local/bin/x86_64-elf-as x86_64-sidos-as
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-gcc
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-cc
ln -s /usr/local/bin/x86_64-elf-ranlib x86_64-sidos-ranlib

liballoc

./newlib-4.1.0/newlib/libc/sys/sidos/crt0.c

file system stuff:
```
let first_file = unsafe { &*(rd_addr as *const FileEntry) };

print_serial!("{:?}\n", first_file);

let filename = core::str::from_utf8(&first_file.filename)
    .unwrap()
    .trim_end();
let ext = core::str::from_utf8(&first_file.ext).unwrap();

let ptr = core::ptr::addr_of!(first_file.size) as *const u32;
let val = unsafe { ptr.read_unaligned() };

print_serial!("{} {} size is {}\n", filename, ext, val);

let mut cluster_addr = get_sector_from_cluster(ds_addr, first_file.cluster_low as usize);

let ptr = kmalloc(10);
unsafe {
    core::ptr::copy(cluster_addr as *mut u8, ptr as *mut u8, 10);
    let c_str = CStr::from_ptr(ptr as *const i8);
    // Convert the CStr to a Rust &str
    let test = c_str.to_str().unwrap().trim();

    print_serial!("{}\n", test);
}

let dir = unsafe { &*((rd_addr + size_of::<FileEntry>()) as *const FileEntry) };
print_serial!("{:?}\n", dir);

let dir_name = core::str::from_utf8(&dir.filename).unwrap();
print_serial!("{}\n", dir_name);

cluster_addr = get_sector_from_cluster(data_sector_addr, dir.cluster_low as usize);

unsafe {
    for i in 0..10 {
        print_serial!("{}", *(cluster_addr as *const u8).offset(i));
    }
}

// let testing = unsafe { &*((rd_addr + 2 * size_of::<FileEntry>()) as *const FileEntry) };
// print_serial!("{:?}\n", testing);
```