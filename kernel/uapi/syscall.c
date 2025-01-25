#include "kernel/uapi/syscall.h"
#include "kernel/defs.h"
#include "kernel/scheduler/proc.h"

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
extern u64 _sys_fork(void);
extern u64 _sys_exit(void);
extern u64 _sys_wait(void);
extern u64 _sys_pipe(void);
extern u64 _sys_read(void);
extern u64 _sys_kill(void);
extern u64 _sys_execve(void);
extern u64 _sys_mmap(void);
extern u64 _sys_fstat(void);
extern u64 _sys_chdir(void);
extern u64 _sys_dup(void);
extern u64 _sys_getpid(void);
extern u64 _sys_sbrk(void);
extern u64 _sys_sleep(void);
extern u64 _sys_uptime(void);
extern u64 _sys_open(void);
extern u64 _sys_write(void);
extern u64 _sys_seek(void);
extern u64 _sys_mknod(void);
extern u64 _sys_unlink(void);
extern u64 _sys_link(void);
extern u64 _sys_mkdir(void);
extern u64 _sys_close(void);
extern u64 _sys_gettimeofday(void);
extern u64 _sys_sysinfo(void);
extern u64 _sys_futimesat(void);
extern u64 _sys_getdents(void);

// An array mapping syscall numbers from syscall.h
// to the function that handles the system call.
static u64 (*syscalls[])(void) = {
    [SYS_fork] = _sys_fork,         [SYS_exit] = _sys_exit,
    [SYS_wait] = _sys_wait,         [SYS_pipe] = _sys_pipe,
    [SYS_read] = _sys_read,         [SYS_kill] = _sys_kill,
    [SYS_execve] = _sys_execve,     [SYS_fstat] = _sys_fstat,
    [SYS_chdir] = _sys_chdir,       [SYS_dup] = _sys_dup,
    [SYS_getpid] = _sys_getpid,     [SYS_sbrk] = _sys_sbrk,
    [SYS_sleep] = _sys_sleep,       [SYS_uptime] = _sys_uptime,
    [SYS_open] = _sys_open,         [SYS_write] = _sys_write,
    [SYS_lseek] = _sys_seek,        [SYS_mknod] = _sys_mknod,
    [SYS_unlink] = _sys_unlink,     [SYS_link] = _sys_link,
    [SYS_mkdir] = _sys_mkdir,       [SYS_close] = _sys_close,
    [SYS_mmap] = _sys_mmap,         [SYS_gettimeofday] = _sys_gettimeofday,
    [SYS_sysinfo] = _sys_sysinfo,   [SYS_futimesat] = _sys_futimesat,
    [SYS_getdents] = _sys_getdents, [SYS_shutdown] = (u64(*)(void))shutdown};

void syscall_impl(void) {
  u64          num;
  struct proc* p = myproc();

  num = p->trapframe->a7;
  if (num > 0 && num < NELEM(syscalls) && syscalls[num]) {
    // Use num to lookup the system call function for num, call it,
    // and store its return value in p->trapframe->a0
    p->trapframe->a0 = syscalls[num]();
    // all syscalls must return. even SYS_exit
  } else {
    printf("%d %s: unknown sys call %lu\n", p->pid, p->name, num);
    p->trapframe->a0 = -1;
  }
}
