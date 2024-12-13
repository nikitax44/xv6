#include "syscall.h"
#include "kernel/defs.h"
#include "kernel/types.h"
#include "scheduler/proc.h"
#include <string.h>

// Fetch the u64 at addr from the current process.
int fetchaddr(u64 addr, u64* ip) {
  struct proc* p = myproc();
  if (addr >= p->sz ||
      addr + sizeof(u64) > p->sz) { // both tests needed, in case of overflow
    return -1;
  }
  if (copyin(p->pagetable, (char*)ip, addr, sizeof(*ip)) != 0) {
    return -1;
  }
  return 0;
}

// Fetch the nul-terminated string at addr from the current process.
// Returns length of string, not including nul, or -1 for error.
int fetchstr(u64 addr, char* buf, int max) {
  struct proc* p = myproc();
  if (copyinstr(p->pagetable, buf, addr, max) < 0) {
    return -1;
  }
  return strlen(buf);
}

static u64 argraw(int n) {
  struct proc* p = myproc();
  switch (n) {
  case 0:
    return p->trapframe->a0;
  case 1:
    return p->trapframe->a1;
  case 2:
    return p->trapframe->a2;
  case 3:
    return p->trapframe->a3;
  case 4:
    return p->trapframe->a4;
  case 5:
    return p->trapframe->a5;
  default:
    panic("argraw");
  }
}

// Fetch the nth 32-bit system call argument.
void argint(int n, int* ip) { *ip = argraw(n); }

// Retrieve an argument as a pointer.
// Doesn't check for legality, since
// copyin/copyout will do that.
void argaddr(int n, u64* ip) { *ip = argraw(n); }

// Fetch the nth word-sized system call argument as a null-terminated string.
// Copies into buf, at most max.
// Returns string length if OK (including nul), -1 if error.
int argstr(int n, char* buf, int max) {
  u64 addr;
  argaddr(n, &addr);
  return fetchstr(addr, buf, max);
}

// Prototypes for the functions that handle system calls.
extern u64 sys_fork(void);
extern u64 sys_exit(void);
extern u64 sys_wait(void);
extern u64 sys_pipe(void);
extern u64 sys_read(void);
extern u64 sys_kill(void);
extern u64 sys_execve(void);
extern u64 sys_mmap(void);
extern u64 sys_fstat(void);
extern u64 sys_chdir(void);
extern u64 sys_dup(void);
extern u64 sys_getpid(void);
extern u64 sys_sbrk(void);
extern u64 sys_sleep(void);
extern u64 sys_uptime(void);
extern u64 sys_open(void);
extern u64 sys_write(void);
extern u64 sys_seek(void);
extern u64 sys_mknod(void);
extern u64 sys_unlink(void);
extern u64 sys_link(void);
extern u64 sys_mkdir(void);
extern u64 sys_close(void);
extern u64 sys_gettimeofday(void);
extern u64 sys_sysinfo(void);
extern u64 sys_futimesat(void);
extern u64 sys_getdents(void);

// An array mapping syscall numbers from syscall.h
// to the function that handles the system call.
static u64 (*syscalls[])(void) = {
    [SYS_fork] = sys_fork,         [SYS_exit] = sys_exit,
    [SYS_wait] = sys_wait,         [SYS_pipe] = sys_pipe,
    [SYS_read] = sys_read,         [SYS_kill] = sys_kill,
    [SYS_execve] = sys_execve,     [SYS_fstat] = sys_fstat,
    [SYS_chdir] = sys_chdir,       [SYS_dup] = sys_dup,
    [SYS_getpid] = sys_getpid,     [SYS_sbrk] = sys_sbrk,
    [SYS_sleep] = sys_sleep,       [SYS_uptime] = sys_uptime,
    [SYS_open] = sys_open,         [SYS_write] = sys_write,
    [SYS_lseek] = sys_seek,        [SYS_mknod] = sys_mknod,
    [SYS_unlink] = sys_unlink,     [SYS_link] = sys_link,
    [SYS_mkdir] = sys_mkdir,       [SYS_close] = sys_close,
    [SYS_mmap] = sys_mmap,         [SYS_gettimeofday] = sys_gettimeofday,
    [SYS_sysinfo] = sys_sysinfo,   [SYS_futimesat] = sys_futimesat,
    [SYS_getdents] = sys_getdents, [SYS_shutdown] = (u64(*)(void))shutdown};

void syscall(void) {
  u64          num;
  struct proc* p = myproc();

  num = p->trapframe->a7;
  if (num > 0 && num < NELEM(syscalls) && syscalls[num]) {
    // Use num to lookup the system call function for num, call it,
    // and store its return value in p->trapframe->a0
    p->trapframe->a0 = syscalls[num]();
  } else {
    printf("%d %s: unknown sys call %lu\n", p->pid, p->name, num);
    p->trapframe->a0 = -1;
  }
}
