#pragma once
#ifndef __ASSEMBLER__
#include "kernel/types.h"
#define asm __asm__
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-function"

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

// use riscv's sv39 page table scheme.
#define SATP_SV39 (8L << 60)

#define MAKE_SATP(pagetable) (SATP_SV39 | (((u64)pagetable) >> 12))

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
static int intr_get(void) {
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

struct sbiret {
  i64 error;
  u64 value;
};

#define SBI_EXT_HSM_HART_START   0
#define SBI_EXT_HSM_HART_STOP    1
#define SBI_EXT_HSM_HART_STATUS  2
#define SBI_EXT_HSM_HART_SUSPEND 3

enum sbi_ext {
  SBI_EXT_HSM  = 0x48534D,
  SBI_EXT_TIME = 0x54494D45,
};
enum perm_mode { MODE_U = 0, MODE_S = 1, MODE_M = 3 };

static struct sbiret sbi_ecall(enum sbi_ext ext, u64 fid, u64 a0, u64 a1,
                               u64 a2, u64 a3, u64 a4, u64 a5) {
  register u64 a0v asm("a0") = a0;
  register u64 a1v asm("a1") = a1;
  register u64 a2v asm("a2") = a2;
  register u64 a3v asm("a3") = a3;
  register u64 a4v asm("a4") = a4;
  register u64 a5v asm("a5") = a5;
  register u64 a6v asm("a6") = fid;
  register u64 a7v asm("a7") = ext;
  asm("ecall"
      : "+r"(a0v), "+r"(a1v)
      : "r"(a2v), "r"(a3v), "r"(a4v), "r"(a5v), "r"(a6v), "r"(a7v)
      : "memory");

  return (struct sbiret){(i64)a0v, a1v};
}

static struct sbiret sbi_hsm_hart_start(u32 hartid, void start(u64 hartid),
                                        enum perm_mode mode) {
  return sbi_ecall(SBI_EXT_HSM, SBI_EXT_HSM_HART_START, hartid, (u64)start,
                   mode, 0, 0, 0);
}
static struct sbiret sbi_hsm_hart_stop(void) {
  return sbi_ecall(SBI_EXT_HSM, SBI_EXT_HSM_HART_STOP, 0, 0, 0, 0, 0, 0);
}
static struct sbiret sbi_hsm_hart_status(u32 hartid) {
  return sbi_ecall(SBI_EXT_HSM, SBI_EXT_HSM_HART_STATUS, hartid, 0, 0, 0, 0, 0);
}

static struct sbiret sbi_set_timer(u64 abstime) {
  return sbi_ecall(SBI_EXT_TIME, 0, abstime, 0, 0, 0, 0, 0);
}

static void set_timer(u64 abstime) {
#ifdef SBI_ENABLE
  sbi_set_timer(abstime);
#else
  w_stimecmp(abstime);
#endif
}

typedef u64  pte_t;
typedef u64* pagetable_t; // 512 PTEs
#pragma GCC diagnostic pop
#endif // __ASSEMBLER__

#define PGSIZE  4096 // bytes per page
#define PGSHIFT 12   // bits of offset within a page

#define PGROUNDUP(sz)  (((sz) + PGSIZE - 1) & ~(PGSIZE - 1))
#define PGROUNDDOWN(a) (((a)) & ~(PGSIZE - 1))

#define PTE_V (1L << 0) // valid
#define PTE_R (1L << 1)
#define PTE_W (1L << 2)
#define PTE_X (1L << 3)
#define PTE_U (1L << 4) // user can access

// shift a physical address to the right place for a PTE.
#define PA2PTE(pa) ((((u64)pa) >> 12) << 10)

#define PTE2PA(pte) (((pte) >> 10) << 12)

#define PTE_FLAGS(pte) ((pte) & 0x3FF)

// extract the three 9-bit page table indices from a virtual address.
#define PXMASK         0x1FF // 9 bits
#define PXSHIFT(level) (PGSHIFT + (9 * (level)))
#define PX(level, va)  ((((u64)(va)) >> PXSHIFT(level)) & PXMASK)

// one beyond the highest possible virtual address.
// MAXVA is actually one bit less than the max allowed by
// Sv39, to avoid having to sign-extend virtual addresses
// that have the high bit set.
#define MAXVA (1L << (9 + 9 + 9 + 12 - 1))
