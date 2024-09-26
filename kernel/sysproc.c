#include "defs.h"
#include "memlayout.h"
#include "param.h"
#include "proc.h"
#include "riscv.h"
#include "spinlock.h"
#include "types.h"

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
