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

#ifndef NULL
#define NULL 0
#endif

#ifndef true
#define true  1
#define false 0
#endif
