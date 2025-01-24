#pragma once
#include "file/fs.h"
#include "types.h"
#include "util/sleeplock.h"

struct buf {
  bool             valid; // has data been read from disk?
  bool             disk;  // does disk "own" buf?
  u32              dev;
  u32              blockno;
  struct sleeplock lock;
  u32              refcnt;
  struct buf*      prev; // LRU cache list
  struct buf*      next;
  u8               data[BSIZE];
};
