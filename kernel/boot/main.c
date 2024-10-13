#include "kernel/defs.h"

// start() jumps here in supervisor mode on all CPUs.
void kernel_main(void) { scheduler(); }

void init_boot(void) {
  consoleinit();
  printfinit();
  printf("\n");
  printf("xv6 kernel is booting\n");
  printf("\n");

  dumpconf();

  kinit();         // physical page allocator lock. after that `free_pages()==0`
  kvminit();       // create kernel page table. initialize the kalloc
  kvminithart();   // turn on paging
  procinit();      // process table
  trapinit();      // trap vectors
  trapinithart();  // install kernel trap vector
  timerinithart(); // request timer interrupts
  plicinit();      // set up interrupt controller
  plicinithart();  // ask PLIC for device interrupts
  binit();         // buffer cache
  iinit();         // inode table
  fileinit();      // file table
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
