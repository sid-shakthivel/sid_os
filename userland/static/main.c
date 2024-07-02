#include <stdio.h>
#include <stdio.h>
#include <unistd.h>
#include <fcntl.h>
#include <string.h>
#include <stdlib.h>

int main()
{
    printf("Hello World\n");
    // puts("Hello is there anybody there\n");

    // int i = 5;

    // char *ptr = "hello from c\n";
    // int64_t result;
    // asm volatile("mov %3, %%ebx \n\t\
    //     mov %2, %%ecx \n\t\
    //     mov %1, %%edx \n\t\
    //     mov $9, %%eax \n\t\
    //     int $0x80 \n\t\
    //     "
    //              : "=r"(result)
    //              : "r"(13), "m"(ptr), "r"(1));

    // lseek(0, 0, 0);
    stat("hello", NULL);

    for (;;)
    {
    }
}
