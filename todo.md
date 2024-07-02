Remember:
- Bochs breakpoint is xchg bx, bx
- info tab specifies v_addr then p_addr
- To print a value in a packed struct do the following:
```
    let ptr = core::ptr::addr_of!(TSS.privilege_stack_table[0]);
    let val = unsafe { ptr.read_unaligned() };
```

0x1FFF000
0x1ffffff

Now: 
- IPC with queues
- Wix kernel mode window manager
- Fix any potential memory leaks

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
https://github.com/thethumbler/Aquila?tab=readme-ov-file
http://www.jamesmolloy.co.uk/tutorial_html/9.-Multitasking.html
http://www.brokenthorn.com/Resources/OSDev25.html
https://is.muni.cz/el/fi/jaro2018/PB173/um/05_writing_a_very_small_os_kernel/osdev.pdf
http://dmitrysoshnikov.com/compilers/writing-a-memory-allocator/
https://wiki.osdev.org/Brendan%27s_Multi-tasking_Tutorial
https://jmarlin.github.io/wsbe/

Current Tabs:
https://jmarlin.github.io/wsbe/
https://github.com/sid-shakthivel/sid_os/blob/6ccee810148848dfd6251a2f2ff59912bd23eac8/kernel/src/gfx/rect.rs
https://github.com/rust-osdev/ps2-mouse/blob/master/src/lib.rs
https://jmnl.xyz/window-manager/
https://github.com/sid-shakthivel/os64/blob/3b90c4e56d66eef83713607586449404adbbd5d0/kernel/src/page_frame_allocator.rs


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