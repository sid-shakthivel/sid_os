#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

#include "../syscalls/syscalls.h"

static Window *new_window;
static int x_base = 5;
static int y_base = 20;

int evaluate_command(char command[255], int wid);

int main()
{
    new_window = malloc(sizeof(Window));
    new_window->x = 100;
    new_window->y = 100;
    new_window->width = 500;
    new_window->height = 350;
    new_window->name = "Terminal";
    new_window->colour = 0x363636;

    char *prompt = "sidos $ ";

    int wid = create_window(new_window, false);

    printf("new window with wid of %d\n", wid);

    char command[255];
    int count = 0;

    strcpy(command, prompt);
    count = strlen(prompt);

    paint_string(command, wid, x_base, y_base);

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
                    y_base += 20;                   // Move onto next line
                    evaluate_command(command, wid); // Evaluate command
                    memset(command, 0, 255);        // Empty string

                    strcpy(command, prompt);
                    count = strlen(prompt);

                    paint_string(command, wid, x_base, y_base);
                }
                else
                {
                    command[count] = event->character;
                    count++;
                    // printf("the command as we go is %s\n", command);
                    paint_string(command, wid, x_base, y_base);
                }
            }
        }
    }
}

int evaluate_command(char command[255], int wid)
{
    char *offset_command = command + 8;

    char new_command[255];
    strcpy(new_command, offset_command);

    if (strcmp(new_command, "hello") == 0)
    {
        paint_string("Hello there", wid, x_base, y_base);
    }
    else if (strcmp(new_command, "doom") == 0)
    {
        paint_string("Doom runs on sidos!", wid, x_base, y_base);
    }
    else
    {
        paint_string("Unknown command", wid, x_base, y_base);
    }
    y_base += 20;
}