#include "kernel/defs.h"

// start() jumps here in supervisor mode on all CPUs.
void kernel_main() { scheduler(); }

void init_boot() {
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
  plicinit();         // set up interrupt controller
  plicinithart();     // ask PLIC for device interrupts
  binit();            // buffer cache
  iinit();            // inode table
  fileinit();         // file table
  virtio_disk_init(); // emulated hard disk
  userinit();         // first user process
}

void init_other() {
  printf("hart %d starting\n", cpuid());
  kvminithart();  // turn on paging
  trapinithart(); // install kernel trap vector
  plicinithart(); // ask PLIC for device interrupts
}
