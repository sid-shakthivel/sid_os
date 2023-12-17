KERNEL = $(shell pwd)/kernel
USERLAND_MODULE_1 = $(shell pwd)/userland/terminal
USERLAND_MODULE_2 = $(shell pwd)/userland/hello-1.3
USERLAND_MODULE_3 = $(shell pwd)/userland/doomgeneric
SYSCALLS = $(shell pwd)/userland/syscalls

run-qemu: all
	qemu-system-x86_64 -accel hvf -serial stdio -cdrom sid_os.iso

all:
	# Replace filesystem
	# rm -f isodir/modules/fs.img
	# cp res/fs.img isodir/modules

	# # Compile syscalls
	# cd $(SYSCALLS) && make

	# # Userspace modules
	# cd $(USERLAND_MODULE_1) && make

	# # cd $(USERLAND_MODULE_2) && make all

	# cd $(USERLAND_MODULE_3) && make all &&\
	# rm -f ../../isodir/modules/doomgeneric &&\
	# cp doomgeneric ../../isodir/modules

	# Kernel
	cd $(KERNEL) && make run

clean:
	rm -f sid_os.iso
	rm -f kernel.bin
	cd kernel && make clean
	cd modules/program && make clean