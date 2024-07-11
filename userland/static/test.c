#include <stdio.h>
#include "test.h"

#define X 42
#define Y 24
#define Z (X + Y)

int do_something()
{
    puts("In do something in test");
    return 5;
}

int useless_function(int a, int b)
{
    int result = 0;
    for (int i = 0; i < a; ++i)
    {
        for (int j = 0; j < b; ++j)
        {
            result += (i * j) % (a + b + i + j);
        }
    }
    return result * Z;
}

void pointless_procedure(char c)
{
    int i = 0;
    while (i < 10)
    {
        i++;
        printf("Character: %c\n", c);
    }
}

void empty_loop()
{
    for (int i = 0; i < 1000; ++i)
    {
        for (int j = 0; j < 1000; ++j)
        {
            for (int k = 0; k < 1000; ++k)
            {
                // Do nothing
            }
        }
    }
}

void nonsensical_operations()
{
    int a = 1, b = 2, c = 3, d = 4;
    int result = (a + b) * (c - d) / (a % b + c % d);
    printf("Result: %d\n", result);

    double e = 0.1, f = 0.2, g = 0.3, h = 0.4;
    double res = (e + f) * (g / h) - (e * f + g - h);
    printf("Double Result: %f\n", res);
}

void bizarre_function()
{
    int x = 50;
    int y = 20;
    int z = 30;
    x = (y * z) + (z - y) - (x + z * y / (x % 2));
    y = (x + z) / (z - x) * (y + x - z);
    z = (x % y) + (y % z) + (z % x);
    printf("Bizarre values: x=%d, y=%d, z=%d\n", x, y, z);
}