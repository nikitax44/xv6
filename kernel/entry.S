        # qemu -kernel loads the kernel at 0x80200000
        # OpenSBI causes boot hart (i.e. CPU) to jump there in S-mode.
        # kernel.ld causes the following code to
        # be placed at 0x80200000.
#include "riscv.h"
.section .entry
.global _entry
.global _entry_other
_entry:
        mv tp, a0
        # set up a stack for C.
        # stack0 is declared in start.c,
        # with a PGSIZE-byte stack per HART.
        # sp = stack0 + PGSIZE
        la sp, stack0
        li t0, PGSIZE
        add sp, sp, t0

        mv a0, a1
        call start_boot
spin:
        j spin

        # all other harts will go there as mandated in start_boot()
_entry_other:
        mv tp, a0

        # $sp=other_stack_arr[$tp]+PGSIZE
        ld t0, other_stack_arr
        slli t1, tp, 3
        add t0, t0, t1
        ld sp, (t0)
        li t0, PGSIZE
        add sp, sp, t0

        call start_other
        j spin
