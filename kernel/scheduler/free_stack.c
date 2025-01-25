#include "kernel/scheduler/free_stack.h"
#include "kernel/util/spinlock.h"

struct {
  u64             max_sz;
  u64             sz;
  u64             mapped;
  u32*            buffer;
  struct spinlock lock;
} stack_storage;

void init_stack_storage(void) {
  initlock(&stack_storage.lock, "free_lock");
  if ((stack_storage.buffer = kalloc()) == 0) {
    panic("kalloc");
  }
  stack_storage.mapped = 0;
  stack_storage.max_sz = START_STACK_SIZE;
  stack_storage.sz     = START_STACK_SIZE;
  for (u64 i = 0; i < START_STACK_SIZE; ++i) {
    stack_storage.buffer[i] = START_STACK_SIZE - i - 1;
  }
}

// Return index of inmuped stack in kernel memory
// Return -1 if error occur
int stack_storage_pop(void) {
  acquire(&stack_storage.lock);
  if (stack_storage.sz == 0) {
    u32* new_buffer = kalloc();
    if (new_buffer == 0) {
      release(&stack_storage.lock);
      return -1;
    }
    for (u64 i = 0; i < stack_storage.max_sz; ++i) {
      new_buffer[i] = 2 * stack_storage.max_sz - i - 1;
    }
    stack_storage.sz = stack_storage.max_sz;
    kfree(stack_storage.buffer);
    stack_storage.max_sz *= 2;
    stack_storage.buffer = new_buffer;
    release(&stack_storage.lock);
    return -1;
  }
  int res = stack_storage.buffer[--stack_storage.sz];
  if (map_stack(res) != 0) {
    stack_storage.sz++;
    stack_storage.mapped--;
    res = -1;
  }
  stack_storage.mapped++;
  release(&stack_storage.lock);
  return res;
}

void stack_storage_push(u32 val) {
  acquire(&stack_storage.lock);
  unmap_stack(val);
  stack_storage.mapped--;
  stack_storage.buffer[stack_storage.sz++] = val;
  release(&stack_storage.lock);
}
