/*
    Framebuffer is portion of RAM which contains a bitmap which maps to the display (pixels)
    GRUB sets the correct video mode before loading the kernel as specified within the multiboot header
    Pitch is number of bytes per row, BPP is bit depth
    Rectangles are arranged like this:
        Top
    Left    Right
        Bottom
    The order of windows is maintained through the stack in which the top most window is at the front and the bottom window is at the back
    Clipping is a method to enable/disable rendering of certain areas by only rendering the topmost pixels in which overlapping regions are not rendered
    A dirty rectangle list is a way to keep track of regions of the screen which need to be repainted which can be used upon the dragging of windows
    PSF(PC Screen Font) fonts consist of header, font, and unicode information
    Glyphs are bitmaps of 8*16

    Each window contains a buffer of it's internal state in which work is completed upon
    The frontbuffer is written to through each window buffer
*/