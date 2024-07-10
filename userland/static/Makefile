# Musl

CC = x86_64-sidos-gcc

LINKER_FILE = $(shell pwd)/musl_linker.ld
LIB_HEADERS = $(shell pwd)/../musl/sidos-out/include
LIBC = $(shell pwd)/../musl/sidos-out/lib/libc.a
LIBS = $(shell pwd)/../musl/sidos-out/lib

CFLAGS = -no-pie -static-libgcc -ffreestanding -nostdlib -fno-builtin -fno-stack-protector -Wall -Wextra -O0 -D_POSIX_SOURCE -I${LIB_HEADERS}
LINKER_FLAGS = -no-pie -static-libgcc -nostdlib -z max-page-size=4096

all: main.o
	$(CC) -T ${LINKER_FILE} $(LINKER_FLAGS) -o static main.o -L$(LIBS) -lgcc $(LIBC) && \
	rm -f ../../isodir/modules/static && \
	mv static ../../isodir/modules

main.o: main.c
	$(CC) -c main.c -o main.o $(CFLAGS)

# PDCLib

# CC = x86_64-sidos-gcc
# LINKER_FILE = $(shell pwd)/musl_linker.ld

# LIBGCC = ../../toolchain/prefix/lib/gcc/i686-elf/12.1.0/libgcc.a

# MINLIBC_INCLUDE = -I ../minlibc/include
# MINLIBC = ../minlibc/minlibc.a

# RAW_CFLAGS = -ffreestanding -nostdlib -O0
# CFLAGS = ${MINLIBC_INCLUDE} ${RAW_CFLAGS} -Wall -Wextra
# LINK_FLAGS = -nostdlib -O2 ${MINLIBC}

# all: main.o
# 	$(CC) -T ${LINKER_FILE} -o static main.o ${LINK_FLAGS} && \
# 	rm -f ../../isodir/modules/static && \
# 	mv static ../../isodir/modules

# main.o: main.c
# 	$(CC) -c main.c -o main.o ${CFLAGS}

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

# Assembly

# ASSEMBLER = nasm
# ASSEMBLER_FLAGS =  -f elf64

# ASSEMBLY_SOURCES := $(wildcard */*.asm) $(wildcard *.asm)
# ASSEMBLY_OBJECTS := $(patsubst %.asm, %.o, $(ASSEMBLY_SOURCES))

# LINKER_FILE = ./linker.ld
# OUPTUT = program

# all: $(ASSEMBLY_OBJECTS)
# 	ld -m elf_x86_64 -o program -T ${LINKER_FILE} ${ASSEMBLY_OBJECTS} -z max-page-size=4096
# 	rm -f ../../isodir/modules/program
# 	mv program ../../isodir/modules

# $(ASSEMBLY_OBJECTS): $(ASSEMBLY_SOURCES)
# 	$(ASSEMBLER) $(ASSEMBLER_FLAGS) $(patsubst %.o, %.asm, $@) -o $@

# clean:
# 	rm -f ${OUTPUT}

clean: 
	rm -f main.o