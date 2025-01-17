#pragma once
#include "kernel/types.h"

struct file;

// map major device number to device functions.
// struct devsw {
//  int (*read)(int, u64, u32);
//  int (*write)(int, u64, u32);
//};
//
// extern struct devsw devsw[];

struct dirent {
  unsigned long d_ino;      /* Inode number */
  char          d_name[24]; /* Filename (null-terminated) */
};