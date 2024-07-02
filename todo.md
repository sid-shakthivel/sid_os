Remember:
- Bochs breakpoint is xchg bx, bx
- info tab specifies v_addr then p_addr
- To print a value in a packed struct do the following:
```
    let ptr = core::ptr::addr_of!(TSS.privilege_stack_table[0]);
    let val = unsafe { ptr.read_unaligned() };
```

Now: 
- Port musl
- Port lua
- IPC with queues
- Fix any potential memory leaks

so dont want rle as not compressed

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

Refactoring
- Window manager stuff
- Make a trait for paint etc
- Want to switch to a userspace window manager
- Continue syscalls

Bugs:
- Dirty rects doesn't work anymore whatsoever
- Need to change the find_first_fit >=
- Mouse when multiple windows

New:
- Sleep syscall thing
- Switch

Later:
- Implement adding/removing list nodes more
- Consider what happens when a window is closed?
- Add font to makefile
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

x86_64-elf-gcc -std=c99 -nostdinc -ffreestanding -fexcess-precision=standard -frounding-math -Wa,--noexecstack -D_XOPEN_SOURCE=700 -I./arch/x86_64 -I./arch/generic -Iobj/src/internal -I./src/include -I./src/internal -Iobj/include -I./include  -Os -pipe -fomit-frame-pointer -fno-unwind-tables -fno-asynchronous-unwind-tables -ffunction-sections -fdata-sections -Wno-pointer-to-int-cast -Werror=implicit-function-declaration -Werror=implicit-int -Werror=pointer-sign -Werror=pointer-arith -Werror=int-conversion -Werror=incompatible-pointer-types -Werror=discarded-qualifiers -Werror=discarded-array-qualifiers -Waddress -Warray-bounds -Wchar-subscripts -Wduplicate-decl-specifier -Winit-self -Wreturn-type -Wsequence-point -Wstrict-aliasing -Wunused-function -Wunused-label -Wunused-variable  -c -o obj/src/linux/module.o src/linux/module.c