#include <stdint.h>
#include <stdbool.h>

// char **environ; /* pointer to array of char * strings that define the current environment variables */

typedef enum
{
    TEXT_MSG,
    CMD_MSG,
    STATUS_MSG,
    ERROR_MSG,
    CONTROL_MSG
} MessageType;

typedef struct
{
    uint64_t sender_pid;
    uint64_t receiver_pid;
    const unsigned char *message; // or const char* if you prefer
    uint64_t length;
    uint64_t m_type;
} Message;

typedef struct Window
{
    uint16_t x;
    uint16_t y;
    uint16_t width;
    uint16_t height;
    unsigned int colour;
    char *name;
} Window;

typedef struct Event
{
    uint8_t flags;
    uint8_t scancode;
    uint8_t character;
    uint16_t mouse_x;
    uint16_t mouse_y;
} Event;

void _exit();
int close(int file);
// int execve(char *name, char **argv, char **env);
// int fork();
// int fstat(int file, struct stat *st);
int getpid();
int isatty(int file);
// int kill(int pid, int sig);
// int link(char *old, char *new);
int open(const char *name, int flags, ...);
// int read(int file, char *ptr, int len);
// int stat(const char *file, struct stat *st);
// clock_t times(struct tms *buf);
// int unlink(char *name);
// int wait(int *status);
// int lseek(int file, long int ptr, int dir);
// int write(int file, char *ptr, int len);
// int gettimeofday(struct timeval *p, void *restrict);
int send_message(Message *message);
Message *receive_message();
int create_window(Window *new_window, bool should_repaint);
Event *get_event();
int paint_string(char *ptr, int wid, int x, int y);
int copy_to_win_buffer(int wid, uint32_t *buffer);

// void *liballoc_alloc(int pages);