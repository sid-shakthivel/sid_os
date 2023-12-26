Remember:
- Bochs breakpoint is xchg bx, bx
- info tab specifies v_addr then p_addr

Refactoring
- PS2 Mouse Things
- When popping in stack/queue should kfree the address
- Need to free memory and clean it up (make it all zero)
- Separate out the bitwise into a separate file
- Replace the free stack within PFA
- Add font to makefile

Bugs:
- Dirty rects when dragging window doesn't fully work
- Kmalloc bug always increases size whenever called (recursive)

New:
- Font support
- Change design of windows
- Syscalls
- Need to extend multitasking capabilities

Later:
- Must consider way to handle handle cr3 register (deep clone to start with)
- Zero the BSS (when we have it)
- Implement adding/removing list nodes

Useful articles:
http://www.jamesmolloy.co.uk/tutorial_html/9.-Multitasking.html
http://www.brokenthorn.com/Resources/OSDev25.html
https://is.muni.cz/el/fi/jaro2018/PB173/um/05_writing_a_very_small_os_kernel/osdev.pdf
http://dmitrysoshnikov.com/compilers/writing-a-memory-allocator/
https://wiki.osdev.org/Brendan%27s_Multi-tasking_Tutorial
https://web.archive.org/web/20170507030615/http://www.trackze.ro/wsbe-complicated-rectangles
