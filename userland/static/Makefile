# Musl
# LINKER_FILE = $(shell pwd)/musl_linker.ld
# LIB_HEADERS = $(shell pwd)/../musl/sidos-out/include
# LIBC = $(shell pwd)/../musl/sidos-out/lib/libc.a
# LIBS = $(shell pwd)/../musl/sidos-out/lib

# Newlib
LINKER_FILE = newlib_linker.ld
LIBC = $(shell pwd)/../libc/build/x86_64-sidos/lib/libc.a
LIBM = $(shell pwd)/../libc/build/x86_64-sidos/lib/libm.a
LIBG = $(shell pwd)/../libc/build/x86_64-sidos/lib/libg.a
LIB_HEADERS = $(shell pwd)/../libc/build/x86_64-sidos/include
SYSCALLS = $(shell pwd)/../syscalls/syscalls.o

CC = x86_64-sidos-gcc
CFLAGS = -no-pie -static-libgcc -nostdlib -fno-builtin -fno-stack-protector -Wall -Wextra -O0 -D_POSIX_SOURCE -I${LIB_HEADERS}
LINKER_FLAGS = -no-pie -static-libgcc -nostdlib -z max-page-size=4096 -nostdlib

TARGET = static

SRCS = $(wildcard *.c)
OBJS = $(SRCS:.c=.o)

install: $(TARGET)
	rm -f ../../isodir/modules/$(TARGET)
	mv $(TARGET) ../../isodir/modules

all: $(TARGET)

# Below is for musl
# $(CC) -T ${LINKER_FILE} $(LINKER_FLAGS) -o $(TARGET) $(OBJS) -L $(LIBS) -lgcc $(LIBC) 
$(TARGET): $(OBJS)
	$(CC) -T ${LINKER_FILE} $(LINKER_FLAGS) -o $(TARGET) $(OBJS) -lgcc $(LIBC) ${LIBM} ${LIBG} $(SYSCALLS)
 
%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

clean: 
	rm -f $(OBJS) $(TARGET)