#include "kernel/hardware/riscv.h"
.section .entry
.global _entry

_entry:
        mv tp, a0
        # set up a stack for C.
        # stack0 is declared in sbi_start.c,
        # with a PGSIZE-byte stack for boot HART.
        # sp = stack0 + PGSIZE
        la sp, stack0
        li t0, PGSIZE
        add sp, sp, t0

        mv a0, a1
        call start_boot
        j spin

        # all other harts will go there as mandated in start_boot()
.global _entry_other
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

spin:
        j spin
