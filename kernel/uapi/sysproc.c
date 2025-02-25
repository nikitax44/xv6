#include "kernel/defs.h"
#include "kernel/errno.h"
#include "kernel/scheduler/proc.h"

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
