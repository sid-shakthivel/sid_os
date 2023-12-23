- Bochs breakpoint is xchg bx, bx
- https://wiki.osdev.org/Programmable_Interval_Timer
- Must consider way to handle handle cr3 register (deep clone to start with)
- Replace the free stack within the pfa
- When popping should really kfree the address
- Zero the BSS (when we have it)
- Implement adding/removing list nodes
- Start the framebuffer
- Proper PS2 keyboard
- Proper PS2 mouse
- Fix the kmalloc bug in which size is always increased whenever called
- info tab specifies v_addr then p_addr

http://www.jamesmolloy.co.uk/tutorial_html/9.-Multitasking.html
http://www.brokenthorn.com/Resources/OSDev25.html
https://is.muni.cz/el/fi/jaro2018/PB173/um/05_writing_a_very_small_os_kernel/osdev.pdf
http://dmitrysoshnikov.com/compilers/writing-a-memory-allocator/
https://web.archive.org/web/20170507030615/http://www.trackze.ro/wsbe-complicated-rectangles