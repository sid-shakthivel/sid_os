- Add scrolling to vga_text mode
- Should there be hashmap of pages a process is using (address + numbers) or something similar
- Add support for grub modules
- Parse the multiboot properly
- Figure whether TSS is needed
- Change interrupts to allow for simple more flags (present, user, etc just more customisable)
- Bochs breakpoint is xchg bx, bx
- https://wiki.osdev.org/Exceptions
- https://wiki.osdev.org/Programmable_Interval_Timer

                    
                                    

    pub fn create_tables(&mut self, mut v_addr: usize, index: usize) {
        if (index == 0) {
            return;
        } else {
            /*
               Need to continue extracting the index for each
            */
            let index: usize = v_addr ;

            let flags = [
                PageFlags::Present,
                PageFlags::Writable,
                PageFlags::UserAccessible,
            ];

            if (self.entries[index].is_unused()) {
                // Create a new table
                let page_frame = PAGE_FRAME_ALLOCATOR
                    .lock()
                    .alloc_page_frame()
                    .expect("Ran out of memory") as usize;
                PAGE_FRAME_ALLOCATOR.free();

                self.entries[index] = Page::new(page_frame, &flags);
            }

            self.create_tables(v_addr >> 12, index);
        }
    }