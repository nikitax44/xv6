#include "free_stack.h"

#include "../defs.h"
#include "../hardware/memlayout.h"
#include "../util/spinlock.h"
#include <string.h>

struct {
  u64             max_sz;
  u64             sz;
  u64             mapped;
  u32*            buffer;
  struct spinlock lock;
} free_stack;

void init_free_stack(void) {
  initlock(&free_stack.lock, "free_lock");
  if ((free_stack.buffer = kalloc()) == 0) {
    panic("bd_malloc");
  }
  free_stack.mapped = 0;
  free_stack.max_sz = START_STACK_SIZE;
  free_stack.sz     = START_STACK_SIZE;
  for (u64 i = 0; i < START_STACK_SIZE; ++i) {
    free_stack.buffer[i] = START_STACK_SIZE - i - 1;
  }
}

// return -1 if error occur
int free_stack_pop(void) {
  acquire(&free_stack.lock);
  if (free_stack.sz == 0) {
    //    u32* new_buffer = kalloc();
    //    if (new_buffer == 0) {
    //      release(&free_stack.lock);
    //      return -1;
    //    }
    //    for (u64 i = 0; i < free_stack.max_sz; ++i) {
    //      new_buffer[i] = 2 * free_stack.max_sz - i - 1;
    //    }
    //    free_stack.sz = free_stack.max_sz;
    //    kfree(free_stack.buffer);
    //    free_stack.max_sz *= 2;
    //    free_stack.buffer = new_buffer;
    release(&free_stack.lock);
    return -1;
  }
  int res = free_stack.buffer[--free_stack.sz];
  if (map_stack(res) != 0) {
    free_stack.sz++;
    free_stack.mapped--;
    res = -1;
  }
  free_stack.mapped++;
  release(&free_stack.lock);
  return res;
}

void free_stack_push(u32 val) {
  acquire(&free_stack.lock);
  unmap_stack(val);
  free_stack.mapped--;
  free_stack.buffer[free_stack.sz++] = val;
  release(&free_stack.lock);
}
