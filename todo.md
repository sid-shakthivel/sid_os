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

Need to do multitasking rn

