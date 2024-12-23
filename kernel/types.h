#pragma once
typedef char  i8;
typedef short i16;
typedef int   i32;
typedef long  i64;

typedef unsigned char  u8;
typedef unsigned short u16;
typedef unsigned int   u32;
typedef unsigned long  u64;

typedef i64 isize;
typedef u64 usize;

typedef const i8* str;
typedef usize     pde_t;
typedef u8 bool;

typedef i64 time_t;
typedef i64 suseconds_t;

struct timeval {
  time_t      tv_sec;  /* seconds */
  suseconds_t tv_usec; /* and microseconds */
};

#ifndef NULL
#define NULL ((void*)0)
#endif

#define true  ((bool)1)
#define false ((bool)0)

#define MIN(a, b) ((a) <= (b) ? (a) : (b))
#define MAX(a, b) ((a) >= (b) ? (a) : (b))
