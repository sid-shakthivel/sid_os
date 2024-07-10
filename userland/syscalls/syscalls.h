#include <stdint.h>

// char **environ; /* pointer to array of char * strings that define the current environment variables */

void _exit();
int close(int file);
int execve(char *name, char **argv, char **env);
int fork();
// int fstat(int file, struct stat *st);
int getpid();
int isatty(int file);
int kill(int pid, int sig);
int link(char *old, char *new);
int open(const char *name, int flags, ...);
int read(int file, char *ptr, int len);
// int stat(const char *file, struct stat *st);
// clock_t times(struct tms *buf);
int unlink(char *name);
int wait(int *status);
int lseek(int file, int ptr, int dir);
int write(int file, char *ptr, int len);
int gettimeofday(struct timeval *p, void *restrict);

void *liballoc_alloc(int pages);