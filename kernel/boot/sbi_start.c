#include "kernel/defs.h"
#include "kernel/hardware/memlayout.h"
#include "kernel/hardware/riscv.h"
#include "kernel/hardware/sbi.h"
#include "kernel/main.h"

void parse_dtb(u8* ptr);
void for_each_hart(void(u64));

// sbi_entry.S
void _entry_other(u64 hartid);

// implemented below
void spawn_hart(u64 hartid);

// sbi_entry.S jumps here in supervisor mode on boot hart.
void start_boot(u8* dtb) {
  // enable interrupts
  w_sie(r_sie() | SIE_SEIE | SIE_STIE | SIE_SSIE);

  // needs to be parsed before kvm init.
  parse_dtb(dtb);

  init_boot();

  __sync_synchronize();

  for_each_hart(spawn_hart);

  kernel_main();
  panic("main exited on boot hart");
}

void start_other(void) {
  w_sie(r_sie() | SIE_SEIE | SIE_STIE | SIE_SSIE);

  init_other();

  __sync_synchronize();

  kernel_main();
  panic("main exited");
}

typedef __attribute__((aligned(PGSIZE))) struct {
  u8 _[PGSIZE];
} page_t;

// sbi_entry.S needs one static stack.
__attribute__((aligned(16))) u8 stack0[PGSIZE];

void spawn_hart(u64 hartid) {
  if (hartid == cpuid()) {
    return;
  }

  if (hartid * sizeof(u8*) > PGSIZE) {
    panic("allocate stack out of page bounds");
  }

  // will be used for 'static by owning HART
  page_t* stack = kalloc();
  if (stack == NULL) {
    panic("failed to allocate stack");
  }

  struct sbiret result =
      sbi_hsm_hart_start(hartid, _entry_other, ((u64)stack) + PGSIZE);
  if (result.error != 0) {
    printf("failed to start hart %lu: %ld\n", hartid, result.error);
  }
}
