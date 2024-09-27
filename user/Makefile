CFLAGS += -fno-builtin-log -fno-builtin-printf -fno-builtin-fprintf -fno-builtin-vprintf -fno-builtin-putc # name clashes

_initcode.h: initcode.S
	$(CC) $(CFLAGS) -nostdinc -c initcode.S -o initcode.o
	$(LD) $(LDFLAGS) -N -e start -Ttext 0 -o initcode.out initcode.o
	$(OBJCOPY) -S -O binary initcode.out initcode
	xxd -i -n initcode initcode > $@

ULIB = ulib.o usys.o printf.o

_%: %.o $(ULIB)
	$(LD) $(LDFLAGS) -T user.ld -o $@ $^ -lc
	$(OBJDUMP) -S $@ > $*.asm
	$(OBJDUMP) -t $@ | sed '1,/SYMBOL TABLE/d; s/ .* / /; /^$$/d' > $*.sym

usys.o: usys.pl
	perl $< > _usys.S
	$(CC) $(CFLAGS) -c -o $@ _usys.S

# Prevent deletion of intermediate files, e.g. cat.o, after first build, so
# that disk image changes after first build are persistent until clean.  More
# details:
# http://www.gnu.org/software/make/manual/html_node/Chained-Rules.html
.PRECIOUS: %.o

-include *.d

all: $(UPROGS)

clean:
	rm -f *.o *.d *.sym *.asm *.out _* initcode

