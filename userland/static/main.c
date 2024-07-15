#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

#include "../syscalls/syscalls.h"

static Window *new_window;
static int x_base = 5;
static int y_base = 20;

int evaluate_command(char command[255])
{
    if (strcmp(command, "hello") == 0)
    {
        paint_string("Hello there user", 0, x_base, y_base);
    }
    else if (strcmp(command, "doom") == 0)
    {
        paint_string("Doom runs on sidos!", 0, x_base, y_base);
    }
    else
    {
        paint_string("Unknown command", 0, x_base, y_base);
    }
    y_base += 20;
}

int main()
{
    new_window = malloc(sizeof(Window));
    new_window->x = 100;
    new_window->y = 100;
    new_window->width = 400;
    new_window->height = 300;
    new_window->name = "Terminal";
    new_window->colour = 0x363636;

    int wid = create_window(new_window, false);

    char command[255];
    int count = 0;

    for (;;)
    {
        // Get event (contains data of mouse, keyboard, etc)
        Event *event = get_event();

        // Check for keyboard event
        if (event->flags & 0b00000001)
        {
            if (count < 255)
            {
                int keycode = (int)event->character;

                // Check for enter key being pressed and do command otherwise, append to string
                if (event->scancode == 0x1c)
                {
                    y_base += 20;              // Move onto next line
                    evaluate_command(command); // Evaluate command
                    memset(command, 0, 255);   // Empty string
                    count = 0;
                }
                else
                {
                    command[count] = event->character;
                    count++;
                    paint_string(command, 0, x_base, y_base);
                }
            }
        }
    }
}
