#pragma once
#include "kernel/types.h"
#define asm __asm__

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-function"

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

// which hart (core) is this?
static u64 r_mhartid(void) {
  u64 x;
  asm volatile("csrr %0, mhartid" : "=r"(x));
  return x;
}

// Machine Status Register, mstatus

#define MSTATUS_MPP_MASK (3L << 11) // previous mode.
#define MSTATUS_MPP_M    (3L << 11)
#define MSTATUS_MPP_S    (1L << 11)
#define MSTATUS_MPP_U    (0L << 11)
#define MSTATUS_MIE      (1L << 3) // machine-mode interrupt enable.

static u64 r_mstatus(void) {
  u64 x;
  asm volatile("csrr %0, mstatus" : "=r"(x));
  return x;
}

static void w_mstatus(u64 x) { asm volatile("csrw mstatus, %0" : : "r"(x)); }

// machine exception program counter, holds the
// instruction address to which a return from
// exception will go.
static void w_mepc(u64 x) { asm volatile("csrw mepc, %0" : : "r"(x)); }

// Supervisor Status Register, sstatus

#define SSTATUS_SPP  (1L << 8) // Previous mode, 1=Supervisor, 0=User
#define SSTATUS_SPIE (1L << 5) // Supervisor Previous Interrupt Enable
#define SSTATUS_UPIE (1L << 4) // User Previous Interrupt Enable
#define SSTATUS_SIE  (1L << 1) // Supervisor Interrupt Enable
#define SSTATUS_UIE  (1L << 0) // User Interrupt Enable

static u64 r_sstatus(void) {
  u64 x;
  asm volatile("csrr %0, sstatus" : "=r"(x));
  return x;
}

static void w_sstatus(u64 x) { asm volatile("csrw sstatus, %0" : : "r"(x)); }

// Supervisor Interrupt Pending
static u64 r_sip(void) {
  u64 x;
  asm volatile("csrr %0, sip" : "=r"(x));
  return x;
}

static void w_sip(u64 x) { asm volatile("csrw sip, %0" : : "r"(x)); }

// Supervisor Interrupt Enable
#define SIE_SEIE (1L << 9) // external
#define SIE_STIE (1L << 5) // timer
#define SIE_SSIE (1L << 1) // software
static u64 r_sie(void) {
  u64 x;
  asm volatile("csrr %0, sie" : "=r"(x));
  return x;
}

static void w_sie(u64 x) { asm volatile("csrw sie, %0" : : "r"(x)); }

// Machine-mode Interrupt Enable
#define MIE_STIE (1L << 5) // supervisor timer
static u64 r_mie(void) {
  u64 x;
  asm volatile("csrr %0, mie" : "=r"(x));
  return x;
}

static void w_mie(u64 x) { asm volatile("csrw mie, %0" : : "r"(x)); }

// supervisor exception program counter, holds the
// instruction address to which a return from
// exception will go.
static void w_sepc(u64 x) { asm volatile("csrw sepc, %0" : : "r"(x)); }

static u64 r_sepc(void) {
  u64 x;
  asm volatile("csrr %0, sepc" : "=r"(x));
  return x;
}

// Machine Exception Delegation
static u64 r_medeleg(void) {
  u64 x;
  asm volatile("csrr %0, medeleg" : "=r"(x));
  return x;
}

static void w_medeleg(u64 x) { asm volatile("csrw medeleg, %0" : : "r"(x)); }

// Machine Interrupt Delegation
static u64 r_mideleg(void) {
  u64 x;
  asm volatile("csrr %0, mideleg" : "=r"(x));
  return x;
}

static void w_mideleg(u64 x) { asm volatile("csrw mideleg, %0" : : "r"(x)); }

// Machine Trap-Vector Base Address
// low two bits are mode.
static void w_mtvec(u64 x) { asm volatile("csrw mtvec, %0" : : "r"(x)); }

static u64 r_mtvec(void) {
  u64 x;
  asm volatile("csrr %0, mtvec" : "=r"(x));
  return x;
}

// Supervisor Trap-Vector Base Address
// low two bits are mode.
static void w_stvec(u64 x) { asm volatile("csrw stvec, %0" : : "r"(x)); }

static u64 r_stvec(void) {
  u64 x;
  asm volatile("csrr %0, stvec" : "=r"(x));
  return x;
}

// Supervisor Timer Comparison Register
static u64 r_stimecmp(void) {
  u64 x;
  // asm volatile("csrr %0, stimecmp" : "=r" (x) );
  asm volatile("csrr %0, 0x14d" : "=r"(x));
  return x;
}

static void w_stimecmp(u64 x) {
  // asm volatile("csrw stimecmp, %0" : : "r" (x));
  asm volatile("csrw 0x14d, %0" : : "r"(x));
}

// Machine Environment Configuration Register
static u64 r_menvcfg(void) {
  u64 x;
  // asm volatile("csrr %0, menvcfg" : "=r" (x) );
  asm volatile("csrr %0, 0x30a" : "=r"(x));
  return x;
}

static void w_menvcfg(u64 x) {
  // asm volatile("csrw menvcfg, %0" : : "r" (x));
  asm volatile("csrw 0x30a, %0" : : "r"(x));
}

// Physical Memory Protection
static void w_pmpcfg0(u64 x) { asm volatile("csrw pmpcfg0, %0" : : "r"(x)); }

static void w_pmpaddr0(u64 x) { asm volatile("csrw pmpaddr0, %0" : : "r"(x)); }

// supervisor address translation and protection;
// holds the address of the page table.
static void w_satp(u64 x) { asm volatile("csrw satp, %0" : : "r"(x)); }

static u64 r_satp(void) {
  u64 x;
  asm volatile("csrr %0, satp" : "=r"(x));
  return x;
}

// Supervisor Trap Cause
static u64 r_scause(void) {
  u64 x;
  asm volatile("csrr %0, scause" : "=r"(x));
  return x;
}

// Supervisor Trap Value
static u64 r_stval(void) {
  u64 x;
  asm volatile("csrr %0, stval" : "=r"(x));
  return x;
}

// Machine-mode Counter-Enable
static void w_mcounteren(u64 x) {
  asm volatile("csrw mcounteren, %0" : : "r"(x));
}

static u64 r_mcounteren(void) {
  u64 x;
  asm volatile("csrr %0, mcounteren" : "=r"(x));
  return x;
}

// machine-mode cycle counter
static u64 r_time(void) {
  u64 x;
  asm volatile("csrr %0, time" : "=r"(x));
  return x;
}

// enable device interrupts
static void intr_on(void) { w_sstatus(r_sstatus() | SSTATUS_SIE); }

// disable device interrupts
static void intr_off(void) { w_sstatus(r_sstatus() & ~SSTATUS_SIE); }

// are device interrupts enabled?
static bool intr_get(void) {
  u64 x = r_sstatus();
  return (x & SSTATUS_SIE) != 0;
}

static u64 r_sp(void) {
  u64 x;
  asm volatile("mv %0, sp" : "=r"(x));
  return x;
}

// read and write tp, the thread pointer, which xv6 uses to hold
// this core's hartid (core number), the index into cpus[].
static u64 r_tp(void) {
  u64 x;
  asm volatile("mv %0, tp" : "=r"(x));
  return x;
}

static void w_tp(u64 x) { asm volatile("mv tp, %0" : : "r"(x)); }

static u64 r_ra(void) {
  u64 x;
  asm volatile("mv %0, ra" : "=r"(x));
  return x;
}

// flush the TLB.
static void sfence_vma(void) {
  // the zero, zero means flush all TLB entries.
  asm volatile("sfence.vma zero, zero");
}

// flush the TLB for specific address.
static inline void sfence_vma_address(u64 addr) {
  asm volatile("sfence.vma zero, %0" ::"r"(addr));
}

#pragma GCC diagnostic pop
