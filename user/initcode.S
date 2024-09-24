# Initial process that execs /init.
# This code runs in user space.

#include "syscall.h"

# exec(init, argv)
.globl start
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
  .quad init
  .quad 0

# char *envp[] = { 0 };
envp:
  .quad 0
