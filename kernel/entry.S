        # qemu -kernel loads the kernel at 0x80000000
        # and causes each hart (i.e. CPU) to jump there.
        # kernel.ld causes the following code to
        # be placed at 0x80000000.
#include "riscv.h"
.section .entry
.global _entry
_entry:
        csrr tp, mhartid
        # set up a stack for C.
        # stack0 is declared in start.c,
        # with a PGSIZE-byte stack per HART.
        # sp = stack0 + ((hartid+1) * 4096)
        la sp, stack0
        li t0, PGSIZE
        addi t1, tp, 1
        mul t0, t0, t1
        add sp, sp, t0
        call start
spin:
        j spin
