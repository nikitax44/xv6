#pragma once
#include "kernel/types.h"

enum file_type {
  T_FILE = 1,
  T_DIR,
  T_LINK,
  T_FIFO,
  T_DEVICE,
  T_BLOCK,
  T_SOCKET,
};

struct stat {
  u32            dev;   // File system's disk device
  usize          ino;   // Inode number
  enum file_type type;  // Type of file
  u32            nlink; // Number of links to file
  u64            size;  // Size of file in bytes
};
