#include "kernel/hardware/riscv.h"
.section .entry
.global _entry

_entry:
#ifdef SBI_ENABLE
        mv tp, a0
        # set up a stack for C.
        # stack0 is declared in start.c,
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
#else
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
#endif

spin:
        j spin
