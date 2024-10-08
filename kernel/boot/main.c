#include "kernel/defs.h"

// start() jumps here in supervisor mode on all CPUs.
void kernel_main(void) { scheduler(); }

#ifndef SBI_ENABLE
static volatile int started = 0;

void init_boot(void);
void init_other(void);

int main(void) {
  if (cpuid() == 0) {
    init_boot();
    __sync_synchronize();
  } else {
    while (started == 0)
      ;
    __sync_synchronize();
    init_other();
  }

  scheduler();
}
#endif

void init_boot(void) {
  consoleinit();
  printfinit();
  printf("\n");
  printf("xv6 kernel is booting\n");
  printf("\n");

  kinit();            // physical page allocator
  kvminit();          // create kernel page table
  kvminithart();      // turn on paging
  procinit();         // process table
  trapinit();         // trap vectors
  trapinithart();     // install kernel trap vector
  timerinithart();    // request timer interrupts
  plicinit();         // set up interrupt controller
  plicinithart();     // ask PLIC for device interrupts
  binit();            // buffer cache
  iinit();            // inode table
  fileinit();         // file table
  virtio_disk_init(); // emulated hard disk
  userinit();         // first user process
}

void init_other(void) {
  printf("hart %d starting\n", cpuid());
  kvminithart();   // turn on paging
  trapinithart();  // install kernel trap vector
  timerinithart(); // request timer interrupts
  plicinithart();  // ask PLIC for device interrupts
}
