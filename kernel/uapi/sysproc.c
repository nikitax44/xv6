#include "kernel/defs.h"
#include "kernel/errno.h"
#include "kernel/hardware/memlayout.h"
#include "kernel/proc.h"
#include "kernel/sysinfo.h"
#include "kernel/types.h"
#include "kernel/util/spinlock.h"
#include <sys/time.h>

u64 sys_exit(void) {
  int n;
  argint(0, &n);
  exit(n);
  return 0; // not reached
}

u64 sys_getpid(void) { return myproc()->pid; }

u64 sys_fork(void) { return fork(); }

u64 sys_wait(void) {
  u64 p;
  argaddr(0, &p);
  return wait(p);
}

u64 sys_sbrk(void) {
  u64 addr;
  int n;

  argint(0, &n);
  addr = myproc()->sz;
  if (growproc(n) < 0) {
    return -1;
  }
  return addr;
}

u64 sys_sleep(void) {
  int n;
  u32 ticks0;

  argint(0, &n);
  if (n < 0) {
    n = 0;
  }
  acquire(&tickslock);
  ticks0 = ticks;
  while (ticks - ticks0 < (u32)n) {
    if (killed(myproc())) {
      release(&tickslock);
      return -1;
    }
    sleep(&ticks, &tickslock);
  }
  release(&tickslock);
  return 0;
}

u64 sys_kill(void) {
  int pid;

  argint(0, &pid);
  return kill(pid);
}

// return how many clock tick interrupts have occurred
// since start.
u64 sys_uptime(void) {
  u32 xticks;

  acquire(&tickslock);
  xticks = ticks;
  release(&tickslock);
  return xticks;
}

u64 sys_gettimeofday(void) {
  u64          outaddr, tzinfo;
  struct proc* p = myproc();
  argaddr(0, &outaddr);
  argaddr(1, &tzinfo);
  struct timeval time = {0, 0};
  if (copyout(p->pagetable, outaddr, (const u8*)&time, sizeof(time)) < 0) {
    return EFAULT;
  }
  return 0;
}

extern u8 end;

u64 sys_sysinfo(void) {
  u64          outaddr;
  struct proc* p = myproc();
  argaddr(0, &outaddr);
  struct sysinfo info = {
      .uptime    = sys_uptime(),
      .loads     = {0},
      .totalram  = 128 * 1024 * 1024,
      .freeram   = PGSIZE * free_pages(),
      .sharedram = 0,
      .bufferram = 0,
      .totalswap = 0,
      .freeswap  = 0,
      .procs     = 4,
      .totalhigh = 0,
      .freehigh  = 0,
      .mem_unit  = 1,
  };
  if (copyout(p->pagetable, outaddr, (const u8*)&info, sizeof(info)) < 0) {
    return EFAULT;
  }
  return 0;
}

u64 sys_futimesat(void) { return ENOSYS; }
