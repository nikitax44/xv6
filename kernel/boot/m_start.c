#include "kernel/defs.h"
#include "kernel/hardware/riscv.h"
#include "kernel/param.h"
#include "kernel/types.h"

#define WARL_R (1 << 0)
#define WARL_W (1 << 1)
#define WARL_X (1 << 2)

#define WARL_LOCK (1 << 7)

#define WARL_OFF   (0x00 << 3)
#define WARL_TOR   (0x01 << 3)
#define WARL_NA4   (0x10 << 3)
#define WARL_NAPOT (0x11 << 3)

#define STCE         (1L << 63)
#define COUNTEREN_TM (1 << 1)

void dispatch(void);
void timerinit(void);

struct __attribute__((aligned(16))) stack {
  char data[2 * PGSIZE];
};

// m_entry.S needs one stack per CPU.
struct stack stack0[NCPU];

// m_entry.S jumps here in machine mode on stack0.
void start(void) {
  // set M Previous Privilege mode to Supervisor, for mret.
  u64 x = r_mstatus();
  x &= ~MSTATUS_MPP_MASK;
  x |= MSTATUS_MPP_S;
  w_mstatus(x);

  // set M Exception Program Counter to main, for mret.
  // requires gcc -mcmodel=medany
  w_mepc((u64)dispatch);

  // disable paging for now.
  w_satp(0);

  // delegate all interrupts and exceptions to supervisor mode.
  w_medeleg(0xffff);
  w_mideleg(0xffff);
  w_sie(r_sie() | SIE_SEIE | SIE_STIE | SIE_SSIE);

  // configure Physical Memory Protection to give supervisor mode
  // access to all of physical memory.
  w_pmpaddr0(0x3fffffffffffffull);
  w_pmpcfg0(WARL_TOR | WARL_R | WARL_W | WARL_X);

  // allow clock interrupts.
  timerinit();

  // switch to supervisor mode and jump to main().
  asm volatile("mret");
}

void timerinit(void) {
  // enable supervisor-mode timer interrupts.
  w_mie(r_mie() | MIE_STIE);

  // enable the sstc extension (i.e. stimecmp).
  w_menvcfg(r_menvcfg() | STCE);

  // allow supervisor to use stimecmp and time.
  w_mcounteren(r_mcounteren() | COUNTEREN_TM);
}

static volatile bool started = false;

void init_boot(void);
void init_other(void);
void kernel_main(void);

void dispatch(void) {
  if (cpuid() == 0) {
    init_boot();
    __sync_synchronize();
    started = true;
  } else {
    while (!started)
      ;
    __sync_synchronize();
    init_other();
  }

  kernel_main();
}
