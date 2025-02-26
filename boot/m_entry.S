#include "kernel/hardware/memlayout.h"
.section .entry
.global _entry

_entry:
        csrr tp, mhartid
        # set up a stack for C.
        # stack0 is declared in m_start.c,
        # with a 2*PGSIZE-byte stack per HART.
        # sp = stack0 + ((hartid+1) * 2*PGSIZE)
        la sp, stack0
        li t0, 2*PGSIZE
        addi t1, tp, 1
        mul t0, t0, t1
        add sp, sp, t0
        call start
spin:
        j spin
