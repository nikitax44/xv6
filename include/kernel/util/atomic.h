#pragma once
#include "kernel/types.h"

typedef struct {
  u32 value;
} AtomicU32;

static inline u32 atomic_fetch_and_add(AtomicU32* atomic, u32 value) {
  return __sync_fetch_and_add(&atomic->value, value);
}

static inline u32 atomic_lock_test_and_set(AtomicU32* atomic, u32 value) {
  return __sync_lock_test_and_set(&atomic->value, value);
}

static inline void atomic_lock_release(AtomicU32* atomic) {
  __sync_lock_release(&atomic->value);
}

static inline u32 atomic_load(AtomicU32* atomic) {
  return *(volatile u32*)&atomic->value;
}
