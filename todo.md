Remember:
- Bochs breakpoint is xchg bx, bx
- info tab specifies v_addr then p_addr

Refactoring
- PS2 Mouse Things
- Separate out the bitwise into a separate file (do once fix ps2)
- Window manager stuff
- Make a trait for paint etc

Bugs:
- Dirty rects doesn't work anymore whatsoever
- Need to change the find_first_fit >=
- Mouse when multiple windows

New:
- Syscalls
- Need to extend multitasking capabilities

Later:
- Must consider way to handle handle cr3 register (deep clone to start with)
- Zero the BSS (when we have it)
- Implement adding/removing list nodes more
- Consider what happens when a window is closed?
- Add font to makefile
- Add more comments everywhere

Trying to fix the bug:
Paging is correct
It's the allocator thats the problem as switching to PFA fixes the bugs

Useful articles:
http://www.jamesmolloy.co.uk/tutorial_html/9.-Multitasking.html
http://www.brokenthorn.com/Resources/OSDev25.html
https://is.muni.cz/el/fi/jaro2018/PB173/um/05_writing_a_very_small_os_kernel/osdev.pdf
http://dmitrysoshnikov.com/compilers/writing-a-memory-allocator/
https://wiki.osdev.org/Brendan%27s_Multi-tasking_Tutorial
https://jmarlin.github.io/wsbe/