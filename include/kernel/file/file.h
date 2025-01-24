#pragma once
#include "fs.h"
#include "kernel/types.h"
#include "kernel/util/sleeplock.h"

struct file {
  enum { FD_NONE, FD_PIPE, FD_INODE, FD_DEVICE } type;
  int           ref; // reference count
  char          readable;
  char          writable;
  struct pipe*  pipe;  // FD_PIPE
  struct inode* ip;    // FD_INODE and FD_DEVICE
  u32           off;   // FD_INODE
  short         major; // FD_DEVICE
};

#define major(dev)  ((dev) >> 16 & 0xFFFF)
#define minor(dev)  ((dev) & 0xFFFF)
#define mkdev(m, n) ((u32)((m) << 16 | (n)))

// in-memory copy of an inode
struct inode {
  u32              dev;   // Device number
  u32              inum;  // Inode number
  int              ref;   // Reference count
  struct sleeplock lock;  // protects everything below here
  int              valid; // inode has been read from disk?

  short type; // copy of disk inode
  short major;
  short minor;
  short nlink;
  u32   size;
  u32   addrs[NDIRECT + 1];
};

// map major device number to device functions.
struct devsw {
  int (*read)(int, u64, u32);
  int (*write)(int, u64, u32);
};

extern struct devsw devsw[];

#define CONSOLE 1
