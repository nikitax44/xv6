#pragma once

#include "kernel/defs.h"

#define START_STACK_SIZE 1024

void init_stack_storage(void);

int stack_storage_pop(void);

void stack_storage_push(u32);
