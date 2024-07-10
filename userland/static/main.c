#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <stdint.h>
// #include "../syscalls/syscalls.h"

int main()
{
    // printf("Lol\n");
    // puts("Hello is there anybody there");

    // int i = 5;

    // char *ptr = "hello from c\n";
    // int64_t result;

    // asm volatile("xchg %bx, %bx");

    // asm volatile("mov %3, %%ebx \n\t\
    //     mov %2, %%ecx \n\t\
    //     mov %1, %%edx \n\t\
    //     mov $1, %%eax \n\t\
    //     syscall \n\t\
    //     "
    //              : "=r"(result)
    //              : "r"(13), "m"(ptr), "r"(1));

    const char *message = "Hello World\n";
    size_t message_length = 15; // Length of the message, including the newline character

    asm volatile("xchg %bx, %bx");

    // Call the write function
    write(1, message, message_length);

    // const char *msg = "Hello, World!\n";
    // syscall(SYS_write, STDOUT_FILENO, msg, 13);

    for (;;)
    {
    }
}
