R=$(realpath .)
K=$R/kernel
U=$R/user
P=$U/ports

export R K U P

QEMU ?= qemu-system-riscv64

ifeq ($(origin TOOLPREFIX),undefined)
	TOOLPREFIX = $(error either TOOLPREFIX or all of CC, LD, OBJCOPY, OBJDUMP must be set and exported by shell)
endif

ifeq ($(origin NEWLIB),undefined)
	NEWLIB := $(error NEWLIB must be set and exported by shell)
endif


ifeq ($(origin CC),default)
	CC := $(TOOLPREFIX)gcc
endif
ifeq ($(origin LD),default)
	LD := $(TOOLPREFIX)ld
endif
ifeq ($(origin OBJCOPY),default)
	OBJCOPY := $(TOOLPREFIX)objcopy
endif
ifeq ($(origin OBJDUMP),default)
	OBJDUMP := $(TOOLPREFIX)objdump
endif

CFLAGS :=
CFLAGS += -Wall -Werror -Wpedantic -Wextra # enable all warnings and make them errors
CFLAGS += -O -fno-omit-frame-pointer -ggdb -gdwarf-2 # debugging stuff
CFLAGS += -MD # generate .d files for dependency resolution
CFLAGS += -mcmodel=medany -march=rv64g -static # required to properly function
CFLAGS += -I $R -I $(NEWLIB)/include --specs=$(NEWLIB)/lib/nano.specs # includes and default options
# CFLAGS += $(shell $(CC) -fno-stack-protector -E -x c /dev/null >/dev/null 2>&1 && echo -fno-stack-protector)

# Disable PIE when possible (for Ubuntu 16.10 toolchain)
ifneq ($(shell $(CC) -dumpspecs 2>/dev/null | grep -e '[^f]no-pie'),)
	CFLAGS += -fno-pie -no-pie
endif
ifneq ($(shell $(CC) -dumpspecs 2>/dev/null | grep -e '[^f]nopie'),)
	CFLAGS += -fno-pie -nopie
endif

LDFLAGS = -z max-page-size=4096 -L $(NEWLIB)/lib -nostdlib

export CC AS LD OBJCOPY OBJDUMP CFLAGS LDFLAGS



all: $K/kernel fs.img

$P/_%: $P/%.o $U/usys.o $P/fixes.o
	$(LD) $(LDFLAGS) -T $P/app.ld -o $@ $(filter %.o,$^) -lc
	$(OBJDUMP) -S $@ > $P/$*.asm
	$(OBJDUMP) -t $@ | sed '1,/SYMBOL TABLE/d; s/ .* / /; /^$$/d' > $P/$*.sym

_exp: $P/dump.c
	$(CC) -o $@ $^

_busybox:
	get-busybox

mkfs/mkfs: mkfs/mkfs.c $K/fs.h $K/param.h
	gcc -Werror -Wall -I. -o mkfs/mkfs mkfs/mkfs.c

UPROGS=\
	$U/_cat\
	$U/_echo\
	$U/_forktest\
	$U/_grep\
	$U/_init\
	$U/_kill\
	$U/_ln\
	$U/_ls\
	$U/_mkdir\
	$U/_rm\
	$U/_sh\
	$U/_stressfs\
	$U/_usertests\
	$U/_grind\
	$U/_wc\
	$U/_zombie\

PORTS=\
	$P/_test\
	$P/_dump\

export UPROGS PORTS

$K/kernel: $U/_initcode.h
	@$(MAKE) -C kernel

$U/%:
	@$(MAKE) -C user $(patsubst $U/%,%,$@)


fs.img: R=.

fs.img: mkfs/mkfs README $(UPROGS) $(PORTS) _exp _busybox
	mkfs/mkfs $@ README $(UPROGS) $(PORTS) _exp _busybox

clean:
	@$(MAKE) -C kernel clean
	@$(MAKE) -C user   clean
	rm -f fs.img mkfs/mkfs .gdbinit

# try to generate a unique GDB port
GDBPORT = $(shell expr `id -u` % 5000 + 25000)
# QEMU's gdb stub command line changed in 0.11
QEMUGDB = $(shell if $(QEMU) -help | grep -q '^-gdb'; \
	then echo "-gdb tcp::$(GDBPORT)"; \
	else echo "-s -p $(GDBPORT)"; fi)

CPUS ?= 3

QEMUOPTS = -machine virt -bios none -kernel $K/kernel -m 128M -smp $(CPUS) -nographic
QEMUOPTS += -global virtio-mmio.force-legacy=false
QEMUOPTS += -drive file=fs.img,if=none,format=raw,id=x0
QEMUOPTS += -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

qemu: $K/kernel fs.img
	$(QEMU) $(QEMUOPTS)

.gdbinit: .gdbinit.tmpl-riscv
	sed "s/:1234/:$(GDBPORT)/" < $^ > $@

qemu-gdb: $K/kernel .gdbinit fs.img
	@echo "*** Now run 'gdb' in another window." 1>&2
	$(QEMU) $(QEMUOPTS) -S $(QEMUGDB)

