#pragma once
#include "atomic.h"
#include "kernel/types.h"

struct rw_lock {
  AtomicU32 readers, writers;
  AtomicU32 lock;
};

void init_rw_lock(struct rw_lock*);

void acquire_read(struct rw_lock*);

void acquire_write(struct rw_lock*);

void release_read(struct rw_lock*);

void release_write(struct rw_lock*);
