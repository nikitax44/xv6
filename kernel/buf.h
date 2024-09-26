#pragma once
#include "fs.h"
#include "sleeplock.h"
#include "types.h"

struct buf {
  int              valid; // has data been read from disk?
  int              disk;  // does disk "own" buf?
  u32              dev;
  u32              blockno;
  struct sleeplock lock;
  u32              refcnt;
  struct buf*      prev; // LRU cache list
  struct buf*      next;
  u8               data[BSIZE];
};
