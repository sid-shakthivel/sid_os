KERNEL = $(shell pwd)/kernel
USERLAND_MODULE_1 = $(shell pwd)/userland/static
USERLAND_MODULE_2 = $(shell pwd)/userland/libforth
USERLAND_MODULE_3 = $(shell pwd)/userland/lua
PROJECT_PATH = $(shell pwd)
SYSCALLS = $(shell pwd)/userland/syscalls

run-qemu: all
	qemu-system-x86_64 -accel hvf -serial stdio -cdrom sid_os.iso

run-bochs: all
	bochs -f bochs/bochsrc.txt -q

all:
	# Replace filesystem
	# rm -f isodir/modules/fs.img
	# cp res/fs.img isodir/modules

	# Compile syscalls
	# cd $(SYSCALLS) && make 

	# Userspace modules
	# cd $(USERLAND_MODULE_1) && make all

	# cd $(USERLAND_MODULE_3) && make generic
	# rm -f $(shell pwd)/isodir/modules/luac
	# cp $(shell pwd)/userland/lua/src/luac $(shell pwd)/isodir/modules

	# Kernel
	cd $(KERNEL) && make run

clean:
	rm -f sid_os.iso
	rm -f kernel.bin
	cd kernel && make clean
	cd modules/program && make clean