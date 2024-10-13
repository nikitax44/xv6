#pragma once
#include "kernel/types.h"

#define DTB_MAGIC 0xD00DFEED

// big-endian
struct fdt_header {
  u32 magic;
  u32 totalsize;
  u32 off_dt_struct;
  u32 off_dt_strings;
  u32 off_mem_rsvmap;
  u32 version;
  u32 last_comp_version;
  u32 boot_cpuid_phys;
  u32 size_dt_strings;
  u32 size_dt_struct;
};

// big-endian
struct fdt_reserve_entry {
  u64 address;
  u64 size;
};

u32 harts(struct fdt_header* dtb);

#define DTB_READ(bits, ptr)                                                    \
  (ptr += bits, bswap##bits(*(volatile u##bits*)(ptr - bits)))
