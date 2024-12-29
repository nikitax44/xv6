#ifndef KERNEL_BITSET_H
#define KERNEL_BITSET_H

#include "defs.h"

#define START_STACK_SIZE 1024

void init_free_stack(void);

int free_stack_pop(void);

void free_stack_push(u32);

#endif
