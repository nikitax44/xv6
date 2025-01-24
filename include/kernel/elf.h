#pragma once
#include "types.h"
// Format of an ELF executable file

#define ELF_MAGIC 0x464C457FU // "\x7FELF" in little endian

// File header
struct elfhdr {
  u32 magic; // must equal ELF_MAGIC
  u8  elf[12];
  u16 type;
  u16 machine;
  u32 version;
  u64 entry;
  u64 phoff;
  u64 shoff;
  u32 flags;
  u16 ehsize;
  u16 phentsize;
  u16 phnum;
  u16 shentsize;
  u16 shnum;
  u16 shstrndx;
};

// Program section header
struct proghdr {
  u32 type;
  u32 flags;
  u64 off;
  u64 vaddr;
  u64 paddr;
  u64 filesz;
  u64 memsz;
  u64 align;
};

// Values for Proghdr type
#define ELF_PROG_LOAD 1

// Flag bits for Proghdr flags
#define ELF_PROG_FLAG_EXEC  1
#define ELF_PROG_FLAG_WRITE 2
#define ELF_PROG_FLAG_READ  4
