#include "syscalls.h"
#include <stdint.h>

int read(int file, char *ptr, int len)
{
    int64_t result;
    asm volatile("mov %3, %%ebx \n\t\
        mov %2, %%ecx \n\t\
        mov %1, %%edx \n\t\
        mov $0, %%eax \n\t\
        int $0x80 \n\t\
        "
                 : "=r"(result)
                 : "r"(len), "m"(ptr), "r"(file));
    return (int)result;
}

int write(int file, char *ptr, int len)
{
    int64_t result;
    asm volatile("mov %3, %%ebx \n\t\
        mov %2, %%ecx \n\t\
        mov %1, %%edx \n\t\
        mov $1, %%eax \n\t\
        int $0x80 \n\t\
        "
                 : "=r"(result)
                 : "r"(len), "m"(ptr), "r"(file));
    return (int)result;
}

int open(const char *name, int flags, ...)
{
    int64_t result;
    asm volatile("mov %0, %%rcx \n\t\
        mov %1, %%rbx \n\t\
        mov $2, %%rax \n\t\
        int $0x80 \n\t\
        "
                 : "=r"(result)
                 : "m"(name), "r"(flags));
    return (int)result;
}

int close(int file)
{
    int64_t result;
    asm volatile("mov %0, %%rbx \n\t\
                 mov $3, %%rax \n\t\
                 int $0x80 \n\t\
                 "
                 : "=r"(result)
                 : "r"(file));
    return (int)result;
}

int lseek(int file, int ptr, int dir)
{
    uint64_t result;
    asm volatile("mov %3, %%ebx \n\t\
        mov %2, %%ecx \n\t\
        mov %1, %%edx \n\t\
        mov $9, %%eax \n\t\
        int $0x80 \n\t\
        "
                 : "=r"(result)
                 : "r"(file), "r"(ptr), "r"(dir));
    return (int)result;
}

void _exit()
{
    asm volatile("mov $56, %rax \n\t\
        int $0x80 \n\t\
        ");
}

int getpid()
{
    int64_t result;
    asm volatile("mov $350, %%rax \n\t\
                 int $0x80 \n\t\
                 "
                 : "=r"(result));
    return (int)result;
}

int isatty(int file)
{
    int64_t result;
    asm volatile("mov %0, %%rbx \n\t\
                 mov $351, %%rax \n\t\
                 int $0x80 \n\t\
                 "
                 : "=r"(result)
                 : "r"(file));
    return (int)result;
}

int send_message(Message *message)
{
    int64_t result;
    asm volatile(
        "mov %[msg_addr], %%rbx \n\t"
        "mov $352, %%rax \n\t"
        "int $0x80 \n\t"
        : "=r"(result)
        : [msg_addr] "r"(message)
        : "rax", "rbx");
    return (int)result;
}

Message *receive_message()
{
    Message *result;
    asm volatile(
        "mov $353, %%rax \n\t"
        "int $0x80 \n\t"
        "mov %%rax, %[result] \n\t"
        : [result] "=r"(result)
        :
        : "rax");
    return result;
}

int create_window(Window *new_window, bool should_repaint)
{

    int64_t result;
    asm volatile("mov %2, %%ebx \n\t\
    mov %1, %%ecx \n\t\
    mov $254, %%eax \n\t\
    int $0x80 \n\t\
    "
                 : "=r"(result)
                 : "r"((int)should_repaint), "m"(new_window));
    return (int)result;
}

Event *get_event()
{
    int64_t result;
    asm volatile("mov $355, %%rax \n\t\
                 int $0x80 \n\t\
                 "
                 : "=r"(result));
    return (Event *)result;
}
