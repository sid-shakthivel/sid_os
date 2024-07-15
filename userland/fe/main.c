#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

#include "../syscalls/syscalls.h"

static Window *new_window;

int main()
{
    new_window = malloc(sizeof(Window));
    new_window->x = 700;
    new_window->y = 400;
    new_window->width = 150;
    new_window->height = 300;
    new_window->name = "File Manager";
    new_window->colour = 0xa5b8df;

    int wid = create_window(new_window, false);

    printf("program 2 with new window with wid of %d\n", wid);

    paint_string("a.txt", wid, 5, 20);
    paint_string("b.txt", wid, 5, 40);

    for (;;)
    {
    }
}
