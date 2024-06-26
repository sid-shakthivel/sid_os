Remember:
- Bochs breakpoint is xchg bx, bx
- info tab specifies v_addr then p_addr
- To print a value in a packed struct do the following:
```
    let ptr = core::ptr::addr_of!(TSS.privilege_stack_table[0]);
    let val = unsafe { ptr.read_unaligned() };
```

Now:
- Can potentially refactor due to null_mut
- Make a dynamic array
- Refactor the priority queue
- Allow for both kernel and user tasks (inheritance)

Refactoring
- PS2 Mouse Things
- Separate out the bitwise into a separate file (do once fix ps2)
- Window manager stuff
- Make a trait for paint etc
- Want to switch to a userspace window manager

Bugs:
- Dirty rects doesn't work anymore whatsoever
- Need to change the find_first_fit >=
- Mouse when multiple windows

New:
- Continue syscalls
- Switch

Later:
- Implement adding/removing list nodes more
- Consider what happens when a window is closed?
- Add font to makefile
- Add more comments everywhere

Useful articles:
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