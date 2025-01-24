#pragma once
#include "kernel/types.h"

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
                               u64 a2) {
  register u64 a0v __asm__("a0") = a0;
  register u64 a1v __asm__("a1") = a1;
  register u64 a2v __asm__("a2") = a2;
  register u64 a6v __asm__("a6") = fid;
  register u64 a7v __asm__("a7") = ext;
  __asm__("ecall"
          : "+r"(a0v), "+r"(a1v)
          : "r"(a2v), "r"(a6v), "r"(a7v)
          : "memory");

  return (struct sbiret){(i64)a0v, a1v};
}

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-function"

static struct sbiret sbi_hsm_hart_start(u32 hartid, void start(u64 hartid),
                                        u64 arg) {
  return sbi_ecall(SBI_EXT_HSM, SBI_EXT_HSM_HART_START, hartid, (u64)start,
                   arg);
}
static struct sbiret sbi_hsm_hart_stop(void) {
  return sbi_ecall(SBI_EXT_HSM, SBI_EXT_HSM_HART_STOP, 0, 0, 0);
}
static struct sbiret sbi_hsm_hart_status(u32 hartid) {
  return sbi_ecall(SBI_EXT_HSM, SBI_EXT_HSM_HART_STATUS, hartid, 0, 0);
}

static struct sbiret sbi_set_timer(u64 abstime) {
  return sbi_ecall(SBI_EXT_TIME, 0, abstime, 0, 0);
}

#pragma GCC diagnostic pop
