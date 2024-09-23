#pragma once
// System call numbers
#define SYS_fork 1
#define SYS_wait 3
#define SYS_pipe 4

#define SYS_exec   7
#define SYS_getpid 11
#define SYS_sleep  13
#define SYS_uptime 14
#define SYS_mknod  17
#define SYS_unlink 18
#define SYS_link   19
#define SYS_mkdir  20

#define SYS_dup   23
#define SYS_chdir 49
#define SYS_close 57
#define SYS_seek  62
#define SYS_read  63
#define SYS_write 64
#define SYS_fstat 80
#define SYS_exit  93
#define SYS_kill  129
#define SYS_sbrk  214
#define SYS_open  430
