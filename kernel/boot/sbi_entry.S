#include "kernel/hardware/riscv.h"
.section .entry

        # a0 contains hartid
        # a1 contains pointer to dtb
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

        # all other harts will go there as mandated in init_hart()
        # a0 contains hartid
        # a1 contains stack pointer
.global _entry_other
_entry_other:
        mv tp, a0
        mv sp, a1
        call start_other

spin:
        j spin
