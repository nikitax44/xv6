#include "kernel/defs.h"
#include "kernel/hardware/riscv.h"
#include "kernel/param.h"
#include "kernel/types.h"

// main.c
void kernel_main(void);
void init_boot(void);
void init_other(void);

void parse_dtb(u8* ptr);
void init_harts(void(u64));

// sbi_entry.S
void _entry_other(u64 hartid);

// implemented below
static void spawn_harts(void);

// sbi_entry.S jumps here in supervisor mode on boot hart.
void start_boot(u8* dtb) {
  // enable interrupts
  w_sie(r_sie() | SIE_SEIE | SIE_STIE | SIE_SSIE);

  // needs to be parsed before kvm init.
  parse_dtb(dtb);

  init_boot();

  __sync_synchronize();

  spawn_harts();

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
// and one more stack for each hart except for the boot one.
page_t** other_stack_arr;

void init_hart(u64 hartid) {
  if (hartid == cpuid()) {
    return;
  }

  if (hartid * sizeof(u8*) > PGSIZE) {
    panic("allocate stack out of page bounds");
  }
  other_stack_arr[hartid] = (page_t*)kalloc();
  if (other_stack_arr[hartid] == NULL) {
    panic("failed to allocate stack");
  }

  struct sbiret result;
  result = sbi_hsm_hart_start(hartid, _entry_other, MODE_S);
  if (result.error != 0) {
    printf("failed to start hart %lu: %ld\n", hartid, result.error);
  }
}

static void spawn_harts(void) {
  other_stack_arr = kalloc();
  if (other_stack_arr == NULL) {
    panic("failed to allocate stack array");
  }
  init_harts(init_hart);
}