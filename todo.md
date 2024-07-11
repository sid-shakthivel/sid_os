Remember:
- Bochs breakpoint is xchg bx, bx
- info tab specifies v_addr then p_addr
- To print a value in a packed struct do the following:
```
    let ptr = core::ptr::addr_of!(TSS.privilege_stack_table[0]);
    let val = unsafe { ptr.read_unaligned() };
```

889 kb
56 kb

Now: 
- Fix exceptions isr stuff
- Fix bochs (its not sse?)
- lseek
- Change the hashmap to *mut T and no pointer for the queue

Refactoring:
- When deleting Set all FAT entries in file's cluster chain to zero
- Rewrite the FileEntry::new() function
- Add back nicer window design

New:
- Sleep syscall
- Continue syscalls
- IPC with message queues
- Events queue for eventual usermode stuff

Useful articles:
https://fejlesztek.hu/create-a-fat-file-system-image-on-linux/
https://github.com/thethumbler/Aquila?tab=readme-ov-file
https://archive.is/KmqPR
http://www.brokenthorn.com/Resources/OSDev25.html
https://is.muni.cz/el/fi/jaro2018/PB173/um/05_writing_a_very_small_os_kernel/osdev.pdf
http://dmitrysoshnikov.com/compilers/writing-a-memory-allocator/
https://wiki.osdev.org/Brendan%27s_Multi-tasking_Tutorial
https://jmarlin.github.io/wsbe/

export CC=x86_64-elf-gcc CFLAGS='--target=x86_64-pc-none-elf -march=x86_64 -DSYSCALL_NO_TLS' LDFLAGS='-fuse-ld=lld' 
export CXX=x86_64-elf-g++
export AR=x86_64-elf-ar
export RANLIB=x86_64-elf-ranlib
export LD=x86_64-elf-ld

export CC=x86_64-sidos-gcc
export ARCH=x86_64
export CROSS_COMPILE=x86_64-sidos-
../configure --target=x86_64-sidos --build=x86_64-sidos --host=x86_64-sidos --prefix=/Users/siddharth/Code/rust/sid_os/userland/musl/sidos-out --disable-sse --enable-debug  CFLAGS='-DSYSCALL_NO_TLS'

ln -s /usr/local/bin/x86_64-elf-ar x86_64-sidos-ar       
ln -s /usr/local/bin/x86_64-elf-as x86_64-sidos-as
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-gcc
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-cc
ln -s /usr/local/bin/x86_64-elf-ranlib x86_64-sidos-ranlib

liballoc

./newlib-4.1.0/newlib/libc/sys/sidos/crt0.c

Where does font= come from? not in makefile

int send_message(int cpid, int pid, char *ptr)
{
    int64_t result;
    asm volatile("mov %1, %%ebx \n\t\
        mov %2, %%ecx \n\t\
        mov %3, %%edx \n\t\
        mov $20, %%eax \n\t\
        int $0x80 \n\t\
        "
                 : "=r"(result)
                 : "r"(cpid), "r"(pid), "m"(ptr));
    return (int)result;
}

```
char *ptr = "hello from c\n";
int64_t result;

asm volatile("mov %3, %%ebx \n\t\
    mov %2, %%ecx \n\t\
    mov %1, %%edx \n\t\
    mov $1, %%eax \n\t\
    syscall \n\t\
    "
                : "=r"(result)
                : "r"(13), "m"(ptr), "r"(1));
```

// NEW
#include <stdio.h>

void nonsensical_operations_b()
{
    int a = 1, b = 2, c = 3, d = 4;
    int result = (a + b) * (c - d) / (a % b + c % d);
    printf("Result: %d\n", result);

    double e = 0.1, f = 0.2, g = 0.3, h = 0.4;
    double res = (e + f) * (g / h) - (e * f + g - h);
    printf("Double Result: %f\n", res);
}

void another_bizarre_function()
{
    int x = 10;
    int y = 20;
    int z = 30;
    x = (y * z) + (z - y) - (x + z * y / (x % 2));
    y = (x + z) / (z - x) * (y + x - z);
    z = (x % y) + (y % z) + (z % x);
    printf("Bizarre values: x=%d, y=%d, z=%d\n", x, y, z);
}

void random_operations()
{
    int a = 5, b = 10, c = 15;
    float d = 1.1f, e = 2.2f, f = 3.3f;

    int result = (a * b + c) - (b / a + c % b) * (a - c);
    float res = (d + e) * (f - d) / (e + f) - (d * e / f);

    printf("Random int result: %d\n", result);
    printf("Random float result: %f\n", res);
}

void meaningless_recursion(int depth)
{
    if (depth == 0)
        return;
    meaningless_recursion(depth - 1);
    printf("Recursion depth: %d\n", depth);
}

int dummy_computation(int n)
{
    if (n <= 0)
        return 0;
    int sum = 0;
    for (int i = 1; i <= n; ++i)
    {
        sum += (i * i) - (i / 2) + (i % 3);
    }
    return sum;
}

void pointless_prints()
{
    for (int i = 0; i < 10; ++i)
    {
        for (int j = 0; j < 5; ++j)
        {
            printf("Pointless iteration i=%d, j=%d\n", i, j);
        }
    }
}

void strange_logic()
{
    int x = 7, y = 14, z = 21;
    if ((x > y) && (y < z) || (x == z))
    {
        x = y + z;
    }
    else if ((x < z) || (z > y))
    {
        y = x - z;
    }
    else
    {
        z = x * y;
    }
    printf("Strange logic values: x=%d, y=%d, z=%d\n", x, y, z);
}

void nonsense_control_flow(int n)
{
    switch (n)
    {
    case 1:
        printf("Case 1\n");
        break;
    case 2:
        printf("Case 2\n");
        break;
    case 3:
        printf("Case 3\n");
        break;
    default:
        for (int i = 0; i < n; ++i)
        {
            printf("Default case, iteration %d\n", i);
        }
        break;
    }
}