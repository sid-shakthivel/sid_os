TARGET = x86_64-sid_os-none

R_COMPILER = cargo

ASSEMBLER = nasm
ASSEMBLER_FLAGS = -f elf64 -O0

OBJ := object_files
RUST_LIBRARY := target/$(TARGET)/debug/libkernel.a
LINKER_FILE := src/linker.ld

ASSEMBLY_SOURCES := $(wildcard src/asm/*.asm) $(wildcard *.asm)
ASSEMBLY_OBJECTS := $(patsubst %.asm, $(OBJ)/%.o, $(ASSEMBLY_SOURCES))
FONT := font.o

PROJECT_PATH = $(shell pwd)/../
KERNEL_PATH = $(shell pwd)

all: sid_os.iso

sid_os.iso: kernel.bin
	cd /code; rm -f isodir/boot/kernel.bin; \
	cp /code/kernel/kernel.bin isodir/boot; \
	grub-mkrescue /usr/lib/grub/i386-pc -o sid_os.iso isodir

kernel.bin: $(ASSEMBLY_OBJECTS) 
	ld -n --gc-sections -o kernel.bin -T ${LINKER_FILE} ${ASSEMBLY_OBJECTS} $(RUST_LIBRARY)

$(ASSEMBLY_OBJECTS): $(ASSEMBLY_SOURCES)
	$(ASSEMBLER) $(ASSEMBLER_FLAGS) $(patsubst $(OBJ)/%.o, %.asm, $@) -o $@

clean:
	$(R_COMPILER) cleans

run:
	RUST_TARGET_PATH=$(KERNEL_PATH) $(R_COMPILER) build --target $(TARGET)
	rm -f kernel.bin
	docker run --rm -v $(PROJECT_PATH):/code sid_os/toolchain bash -c "cd code/kernel && make"
