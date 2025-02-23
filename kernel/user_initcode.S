.section .rodata
.globl initcode_start
initcode_start:
# .org 0 # is ignored

# Initial process that execs /init.
# This code runs in user space.

#include "kernel/uapi/syscall.h"
# exec(init, argv)
start:
        la a0, init
        la a1, argv
        la a2, envp
        li a7, SYS_execve
        ecall

# for(;;) exit();
exit:
        li a7, SYS_exit
        ecall
        jal exit

# char init[] = "/init\0";
init:
  .string "/init\0"

.p2align 2
# char *argv[] = { init, 0 };
argv:
  .quad init-start # stores absolute address in kernelspace, so shift it back to zero
  .quad 0

# char *envp[] = { 0 };
envp:
  .quad 0

.globl initcode_end
initcode_end:
