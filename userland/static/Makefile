# Musl

CC = x86_64-sidos-gcc

LINKER_FILE = $(shell pwd)/musl_linker.ld
LIB_HEADERS = $(shell pwd)/../musl/sidos-out/include
LIBC = $(shell pwd)/../musl/sidos-out/lib/libc.a
LIBS = $(shell pwd)/../musl/sidos-out/lib

CFLAGS = -no-pie -static-libgcc -nostdlib -fno-builtin -fno-stack-protector -Wall -Wextra -O0 -D_POSIX_SOURCE -I${LIB_HEADERS}
LINKER_FLAGS = -no-pie -static-libgcc -nostdlib -z max-page-size=4096

TARGET = static

SRCS = $(wildcard *.c)
OBJS = $(SRCS:.c=.o)

all: $(TARGET)
	rm -f ../../isodir/modules/$(TARGET) && \
	mv $(TARGET) ../../isodir/modules

$(TARGET): $(OBJS)
	$(CC) -T ${LINKER_FILE} $(LINKER_FLAGS) -o $(TARGET) $(OBJS) -L $(LIBS) -lgcc $(LIBC) 

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

# Newlib

# CC = x86_64-elf-gcc

# CFLAGS = -ffreestanding -O2 -Wall -Wextra
# LINKER_FLAGS = -ffreestanding -O2 -nostdlib -z max-page-size=4096

# LINKER_FILE = newlib_linker.ld
# LIBC = $(shell pwd)/../libc/build/x86_64-sidos/lib/libc.a
# LIBM = $(shell pwd)/../libc/build/x86_64-sidos/lib/libm.a
# LIBG = $(shell pwd)/../libc/build/x86_64-sidos/lib/libg.a
# LIB_HEADERS = $(shell pwd)/../libc/build/x86_64-sidos/include
# SYSCALLS = $(shell pwd)/../syscalls/syscalls.o

# all: main.o
# 	$(CC) -T ${LINKER_FILE} -no-pie -static-libgcc -o static main.o -nostdlib -z max-page-size=4096 -lgcc ${LIBM} ${LIBC} ${LIBG} $(SYSCALLS) && \
# 	rm -f ../../isodir/modules/static && \
#  	mv static ../../isodir/modules 
 
# main.o: main.c
# 	$(CC) -no-pie -static-libgcc -c main.c -o main.o -ffreestanding -nostdlib -fno-builtin -fno-stack-protector -Wall -Wextra -I ${LIB_HEADERS}

clean: 
	rm -f $(OBJS) $(TARGET)