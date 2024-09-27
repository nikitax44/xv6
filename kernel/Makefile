OBJS = \
  entry.o \
  start.o \
  console.o \
  printf.o \
  uart.o \
  kalloc.o \
  spinlock.o \
  string.o \
  main.o \
  vm.o \
  proc.o \
  swtch.o \
  trampoline.o \
  trap.o \
  syscall.o \
  sysproc.o \
  bio.o \
  fs.o \
  log.o \
  sleeplock.o \
  file.o \
  pipe.o \
  exec.o \
  sysfile.o \
  kernelvec.o \
  plic.o \
  virtio_disk.o

CFLAGS += -ffreestanding
CFLAGS += -fno-builtin-log -fno-builtin-printf -fno-builtin-fprintf -fno-builtin-vprintf -fno-builtin-putc # name clashes

all: kernel

kernel: $(OBJS) kernel.ld
	$(LD) $(LDFLAGS) -T kernel.ld -o kernel $(OBJS) -lc
	$(OBJDUMP) -S kernel > kernel.asm
	$(OBJDUMP) -t kernel | sed '1,/SYMBOL TABLE/d; s/ .* / /; /^$$/d' > kernel.sym

proc.o: $U/_initcode.h

# Prevent deletion of intermediate files, e.g. cat.o, after first build, so
# that disk image changes after first build are persistent until clean.  More
# details:
# http://www.gnu.org/software/make/manual/html_node/Chained-Rules.html
.PRECIOUS: %.o

-include *.d

clean:
	rm -f *.o *.d *.asm *.sym kernel
